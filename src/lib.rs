use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, Token, Ident};
use serde::{Deserialize, Serialize};

#[proc_macro]
pub fn declare_program(input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as DeclareInput);
    
    let idl = match read_idl_file(&input_struct.idl_path, input_struct.idl_version) {
        Ok(idl) => idl,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to read IDL file '{}' as version {}: {}", 
                       input_struct.idl_path, 
                       input_struct.idl_version.unwrap_or(1), 
                       e)
            ).to_compile_error().into();
        }
    };
    
    let generated = generate_program_module(&input_struct.name, &input_struct.id, &idl);
    
    generated.into()
}

fn generate_program_module(module_name: &str, program_id: &Option<String>, idl: &Idl) -> proc_macro2::TokenStream {
    let module_ident = Ident::new(module_name, proc_macro2::Span::call_site());
    
    let instruction_count = idl.instructions.len();
    let instruction_names: Vec<&str> = idl.instructions.iter().map(|i| i.name.as_str()).collect();
    
    let program_id_code = generate_program_id_constant(program_id);
    
    let cpi_functions: Vec<_> = idl.instructions
        .iter()
        .enumerate()
        .map(|(index, instruction)| generate_cpi_function_generic(instruction, index as u8))
        .collect();
    
    quote! {
        pub mod #module_ident {
            use pinocchio::{
                account_info::AccountInfo,
                instruction::{AccountMeta, Instruction},
                cpi::invoke,  
                pubkey::Pubkey,
                ProgramResult,
            };

            pub const MODULE_NAME: &str = #module_name;
            pub const INSTRUCTION_COUNT: usize = #instruction_count;
            pub const INSTRUCTIONS: &[&str] = &[#(#instruction_names),*];
            

            #program_id_code
            
            #(#cpi_functions)*
        }
    }
}

fn generate_program_id_constant(program_id: &Option<String>) -> proc_macro2::TokenStream {
    match program_id {
        Some(id) => {
            quote! {
                pub const PROGRAM_ID: &str = #id;
                
                pub fn program_id() -> Pubkey {

                    [0u8; 32]
                }
            }
        }
        None => {
            quote! {
                /// program ID not specified 
                pub const PROGRAM_ID: &str = "11111111111111111111111111111111";
                
                pub fn program_id() -> Pubkey {
                    [0u8; 32] 
                }
            }
        }
    }
}

fn generate_cpi_function_generic(instruction: &Instruction, discriminant: u8) -> proc_macro2::TokenStream {
    let function_name = syn::Ident::new(&instruction.name, proc_macro2::Span::call_site());
    
    let account_params: Vec<_> = instruction.accounts
        .iter()
        .map(|account| {
            let param_name = syn::Ident::new(&account.name, proc_macro2::Span::call_site());
            quote! { #param_name: &AccountInfo }
        })
        .collect();
    
    let arg_params: Vec<_> = instruction.args
        .iter()
        .map(|arg| {
            let param_name = syn::Ident::new(&arg.name, proc_macro2::Span::call_site());
            quote! { #param_name: Vec<u8> }
        })
        .collect();
    
    let all_params = [account_params, arg_params].concat();
    
    let account_metas: Vec<_> = instruction.accounts
        .iter()
        .map(|account| {
            let param_name = syn::Ident::new(&account.name, proc_macro2::Span::call_site());
            
            match (account.is_mut, account.is_signer) {
                (true, true) => quote! { AccountMeta::writable_signer(#param_name.key()) },
                (true, false) => quote! { AccountMeta::writable(#param_name.key()) },
                (false, true) => quote! { AccountMeta::readonly_signer(#param_name.key()) },
                (false, false) => quote! { AccountMeta::readonly(#param_name.key()) },
            }
        })
        .collect();
    
    let account_infos: Vec<_> = instruction.accounts
        .iter()
        .map(|account| {
            let param_name = syn::Ident::new(&account.name, proc_macro2::Span::call_site());
            quote! { #param_name }
        })
        .collect();

    let instruction_data = if instruction.args.is_empty() {
        quote! { vec![#discriminant] }
    } else {
        let arg_names: Vec<_> = instruction.args
            .iter()
            .map(|arg| {
                let param_name = syn::Ident::new(&arg.name, proc_macro2::Span::call_site());
                quote! { &#param_name }
            })
            .collect();
        
        quote! {
            {
                let mut data = vec![#discriminant];
                #(data.extend(#arg_names);)*
                data
            }
        }
    };
    
    quote! {
        pub fn #function_name(
            #(#all_params),*
        ) -> ProgramResult {
            let instruction = Instruction {
                program_id: &program_id(),
                accounts: &[
                    #(#account_metas),*
                ],
                data: &#instruction_data,
            };
            
            invoke(&instruction, &[#(#account_infos),*])
        }
    }
}

struct DeclareInput {
    name: String,
    id: Option<String>,
    idl_path: String,
    idl_version: Option<u32>,  
}

impl syn::parse::Parse for DeclareInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut id = None;
        let mut idl_path = None;
        let mut idl_version = None;
        
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            
            match key.to_string().as_str() {
                "name" => {
                    let value: LitStr = input.parse()?;
                    name = Some(value.value());
                },
                "id" => {
                    let value: LitStr = input.parse()?;
                    id = Some(value.value());
                },
                "idl_path" => {
                    let value: LitStr = input.parse()?;
                    idl_path = Some(value.value());
                },
                "idl_version" => {
                    let value: syn::LitInt = input.parse()?;
                    idl_version = Some(value.base10_parse::<u32>()?);
                },
                _ => return Err(syn::Error::new_spanned(
                    key, 
                    "Unknown key. Expected 'name', 'id', 'idl_path', or 'idl_version'"
                )),
            }
            
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        
        Ok(DeclareInput {
            name: name.ok_or_else(|| syn::Error::new(input.span(), "Missing 'name' parameter"))?,
            id,
            idl_path: idl_path.ok_or_else(|| syn::Error::new(input.span(), "Missing 'idl_path' parameter"))?,
            idl_version: idl_version.or(Some(1)), // default to version 1
        })
    }
}


#[derive(Debug, Deserialize, Serialize)]
struct IdlV1 {
    name: String,
    instructions: Vec<Instruction>,
    
    #[serde(flatten)]
    other_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IdlV2 {

    metadata: IdlMetadata,
    instructions: Vec<Instruction>,
    
    #[serde(flatten)]
    other_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct Idl {
    name: String,
    instructions: Vec<Instruction>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IdlMetadata {
    name: String,
    version: String,
    spec: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Instruction {
    name: String,
    accounts: Vec<Account>,
    #[serde(default)]  
    args: Vec<Arg>,
    #[serde(flatten)]
    other_fields: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Account {
    name: String,
    
    #[serde(default)]
    #[serde(alias = "isMut")]
    #[serde(alias = "writable")]
    #[serde(alias = "is_mut")]
    #[serde(alias = "mutable")]
    is_mut: bool,
    
    #[serde(default)]
    #[serde(alias = "isSigner")]
    #[serde(alias = "signer")]
    #[serde(alias = "is_signer")]
    #[serde(alias = "signs")]
    is_signer: bool,
    
    #[serde(flatten)]
    other_fields: HashMap<String, serde_json::Value>,
}


#[derive(Debug, Deserialize, Serialize)]
struct Arg {
    name: String,
    #[serde(rename = "type")]
    arg_type: serde_json::Value, 
}

fn read_idl_file(path: &str, version: Option<u32>) -> Result<Idl, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    
    match version.unwrap_or(1) {
        1 => {
            let idl_v1: IdlV1 = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse as V1 IDL: {}", e))?;
            
            Ok(Idl {
                name: idl_v1.name,
                instructions: idl_v1.instructions,
            })
        },
        2 => {
            let idl_v2: IdlV2 = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse as V2 IDL: {}", e))?;
            
            Ok(Idl {
                name: idl_v2.metadata.name,
                instructions: idl_v2.instructions,
            })
        },
        _ => {
            Err(format!("Unsupported IDL version: {}. Supported versions: 1, 2", version.unwrap_or(1)).into())
        }
    }
}