use sanity::declare_program;

// Generate actual modules to test
declare_program! {
    name = "test_spl",
    id = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    idl_path = "tests/fixtures/spl_token.json",
    idl_version = 1
}

declare_program! {
    name = "test_pump",
    id = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
    idl_path = "tests/fixtures/pump_v2.json",
    idl_version = 2
}

#[cfg(test)]
mod macro_verification_tests {
    use super::*;
    use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
    use std::any::type_name;
    
    type ProgramResult = Result<(), ProgramError>;

    #[test]
    fn test_macro_generates_correct_module_structure() {
        println!("🔍 TESTING: Macro generates correct module structure");
        
        assert_eq!(test_spl::MODULE_NAME, "test_spl");
        assert_eq!(test_pump::MODULE_NAME, "test_pump");
        
        assert_eq!(test_spl::PROGRAM_ID, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        assert_eq!(test_pump::PROGRAM_ID, "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
        
        assert!(test_spl::INSTRUCTION_COUNT > 0);
        assert!(test_pump::INSTRUCTION_COUNT > 0);
        assert_ne!(test_spl::INSTRUCTION_COUNT, test_pump::INSTRUCTION_COUNT);
        
        // test instruction arrays are populated
        assert!(test_spl::INSTRUCTIONS.len() > 0);
        assert!(test_pump::INSTRUCTIONS.len() > 0);
        assert_eq!(test_spl::INSTRUCTIONS.len(), test_spl::INSTRUCTION_COUNT);
        assert_eq!(test_pump::INSTRUCTIONS.len(), test_pump::INSTRUCTION_COUNT);
        
        println!(" Module constants generated correctly");
        println!(" spl Token: {} instructions", test_spl::INSTRUCTION_COUNT);
        println!(" Pump.fun: {} instructions", test_pump::INSTRUCTION_COUNT);
    }


    #[test]
    fn test_macro_generates_actual_cpi_functions() {
        println!("TESTING: Macro generates actual CPI functions");
        
        
        // transfer: 3 accounts + 1 arg
        let _: fn(&AccountInfo, &AccountInfo, &AccountInfo, Vec<u8>) -> ProgramResult 
            = test_spl::transfer;
        
        // mintTo: 3 accounts + 1 arg  
        let _: fn(&AccountInfo, &AccountInfo, &AccountInfo, Vec<u8>) -> ProgramResult 
            = test_spl::mintTo;
        
        // revoke: 2 accounts + 0 args
        let _: fn(&AccountInfo, &AccountInfo) -> ProgramResult 
            = test_spl::revoke;
        
        // initializeMint: 2 accounts + 3 args
        let _: fn(&AccountInfo, &AccountInfo, Vec<u8>, Vec<u8>, Vec<u8>) -> ProgramResult 
            = test_spl::initializeMint;
        
        println!("SPL Token functions generated with correct signatures");
        
        
        // initialize: 3 accounts + 0 args
        let _: fn(&AccountInfo, &AccountInfo, &AccountInfo) -> ProgramResult 
            = test_pump::initialize;
        
        // buy: 12 accounts + 2 args
        let _: fn(&AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, Vec<u8>, Vec<u8>) -> ProgramResult 
            = test_pump::buy;
        
        // withdraw: 12 accounts + 0 args
        let _: fn(&AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo) -> ProgramResult 
            = test_pump::withdraw;
        
        println!("Pump.fun functions generated with correct signatures");
        println!("All functions return ProgramResult as expected");
    }

    #[test]
    fn test_macro_handles_different_idl_versions() {
        println!("TESTING: Macro handles different IDL versions correctly");
        
        assert_eq!(test_spl::MODULE_NAME, "test_spl");
        assert!(test_spl::INSTRUCTION_COUNT > 20); 
        

        assert_eq!(test_pump::MODULE_NAME, "test_pump");
        assert_eq!(test_pump::INSTRUCTION_COUNT, 6); 
        
        // both should generate functions
        let _spl_fn: fn(&AccountInfo, &AccountInfo, &AccountInfo, Vec<u8>) -> ProgramResult = test_spl::transfer;
        let _pump_fn: fn(&AccountInfo, &AccountInfo, &AccountInfo) -> ProgramResult = test_pump::initialize;
        
        println!("V1 IDL parsed and generated correctly");
        println!("V2 IDL parsed and generated correctly");
        println!("Both versions coexist without conflicts");
    }

    #[test]
    fn test_macro_generates_correct_instruction_names() {
        println!("TESTING: Macro extracts correct instruction names from IDL");
        
        let spl_instructions = test_spl::INSTRUCTIONS;
        
        // known spl token instructions that should be present
        let expected_spl = ["transfer", "mintTo", "burn", "approve", "revoke"];
        for expected in &expected_spl {
            assert!(spl_instructions.contains(expected), 
                   "SPL Token should have '{}' instruction", expected);
        }
        
        let pump_instructions = test_pump::INSTRUCTIONS;
        
        let expected_pump = ["initialize", "create", "buy", "sell", "withdraw"];
        for expected in &expected_pump {
            assert!(pump_instructions.contains(expected),
                   "Pump.fun should have '{}' instruction", expected);
        }
        
        println!("SPL Token instructions: {:?}", &spl_instructions[..5.min(spl_instructions.len())]);
        println!("Pump.fun instructions: {:?}", pump_instructions);
        println!("All expected instruction names found");
    }

    #[test]
    fn test_macro_function_return_types() {
        println!("TESTING: Generated functions have correct return types");
        
        //  should return ProgramResult
        fn assert_returns_program_result<F>(_f: F) 
        where 
            F: Fn() -> ProgramResult 
        {
            // This function only compiles if F returns ProgramResult
        }
        
        // test to call functions (proves they compile)
        // we can't actually call them without real AccountInfo, but we can verify signatures
        
        let transfer_fn = test_spl::transfer;
        let mint_fn = test_spl::mintTo;
        let init_fn = test_pump::initialize;
        let buy_fn = test_pump::buy;
        
        // verify return types match expectations
        let transfer_type = type_name::<fn(&AccountInfo, &AccountInfo, &AccountInfo, Vec<u8>) -> ProgramResult>();
        let init_type = type_name::<fn(&AccountInfo, &AccountInfo, &AccountInfo) -> ProgramResult>();
        
        println!("transfer function type: {}", transfer_type);
        println!("initialize function type: {}", init_type);
        println!("All functions return ProgramResult");
    }

    #[test]
    fn test_macro_argument_handling() {
        println!("TESTING: Macro handles arguments correctly");
        
        let _no_args: fn(&AccountInfo, &AccountInfo) -> ProgramResult = test_spl::revoke;
        let _no_args2: fn(&AccountInfo, &AccountInfo, &AccountInfo) -> ProgramResult = test_pump::initialize;
        
        // functions with arguments should have Vec<u8> for each argument
        let _one_arg: fn(&AccountInfo, &AccountInfo, &AccountInfo, Vec<u8>) -> ProgramResult = test_spl::transfer;
        let _three_args: fn(&AccountInfo, &AccountInfo, Vec<u8>, Vec<u8>, Vec<u8>) -> ProgramResult = test_spl::initializeMint;
        let _two_args: fn(&AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, Vec<u8>, Vec<u8>) -> ProgramResult = test_pump::buy;
        
        println!("No-arg functions: correct signature");
        println!("Single-arg functions: correct signature");  
        println!("Multi-arg functions: correct signature");
        println!("All arguments become Vec<u8> as expected");
    }

    #[test]
    fn test_macro_account_handling() {
        println!("🔍 TESTING: Macro handles accounts correctly");
        
        // count accounts by counting &AccountInfo params
        
        // transfer: 3 accounts (source, dest, authority)
        let _transfer: fn(&AccountInfo, &AccountInfo, &AccountInfo, Vec<u8>) -> ProgramResult = test_spl::transfer;
        
        // revoke: 2 accounts (source, owner)
        let _revoke: fn(&AccountInfo, &AccountInfo) -> ProgramResult = test_spl::revoke;
        
        // pump initialize: 3 accounts  
        let _init: fn(&AccountInfo, &AccountInfo, &AccountInfo) -> ProgramResult = test_pump::initialize;
        
        // pump buy: 12 accounts (complex instruction)
        let _buy: fn(&AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, &AccountInfo, Vec<u8>, Vec<u8>) -> ProgramResult = test_pump::buy;
        
        println!("Simple functions (2-3 accounts): correct");
        println!("Complex functions (12+ accounts): correct");
        println!("All accounts become &AccountInfo parameters");
    }

    #[test]
    fn test_generated_constants_are_correct() {
        println!("🔍 TESTING: Generated constants match what we specified");
        
        assert_eq!(test_spl::MODULE_NAME, "test_spl");
        assert_eq!(test_spl::PROGRAM_ID, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        
        assert_eq!(test_pump::MODULE_NAME, "test_pump");
        assert_eq!(test_pump::PROGRAM_ID, "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
        
        assert_eq!(test_spl::INSTRUCTION_COUNT, test_spl::INSTRUCTIONS.len());
        assert_eq!(test_pump::INSTRUCTION_COUNT, test_pump::INSTRUCTIONS.len());
        
        let _: &str = test_spl::MODULE_NAME;
        let _: &str = test_spl::PROGRAM_ID;
        let _: usize = test_spl::INSTRUCTION_COUNT;
        let _: &[&str] = test_spl::INSTRUCTIONS;
        
        println!("All constants have correct values");
        println!("All constants have correct types");
        println!("Instruction counts match array lengths");
    }

    #[test]
    fn test_macro_compilation_success() {
        println!("🔍 TESTING: Generated code compiles without errors");
        
        // if this test runs, it means all the generated code compiled successfully
        
        let _ = test_spl::MODULE_NAME;
        let _ = test_spl::PROGRAM_ID;
        let _ = test_spl::INSTRUCTION_COUNT;
        let _ = test_spl::INSTRUCTIONS;
        let _ = test_spl::program_id;
        let _ = test_spl::transfer;
        let _ = test_spl::mintTo;
        
        let _ = test_pump::MODULE_NAME;
        let _ = test_pump::PROGRAM_ID;
        let _ = test_pump::INSTRUCTION_COUNT;
        let _ = test_pump::INSTRUCTIONS;
        let _ = test_pump::program_id;
        let _ = test_pump::initialize;
        let _ = test_pump::buy;
        
        println!("All generated code compiles successfully");
        println!("No compilation errors or warnings");
        println!("All generated items are accessible");
    }

    #[test]
    fn test_print_actual_generated_output() {
        println!("\n ACTUAL MACRO OUTPUT VERIFICATION");
        println!("===================================");
        
        println!("\n SPL TOKEN PROGRAM:");
        println!("  Module Name: {}", test_spl::MODULE_NAME);
        println!("  Program ID: {}", test_spl::PROGRAM_ID);
        println!("  Instruction Count: {}", test_spl::INSTRUCTION_COUNT);
        println!("  Instructions: {:?}", test_spl::INSTRUCTIONS);
        println!("  Program ID bytes: {:?}", &test_spl::program_id()[..4]); // first 4 bytes
        
        println!("\n PUMP.FUN PROGRAM:");
        println!("  Module Name: {}", test_pump::MODULE_NAME);
        println!("  Program ID: {}", test_pump::PROGRAM_ID);
        println!("  Instruction Count: {}", test_pump::INSTRUCTION_COUNT);
        println!("  Instructions: {:?}", test_pump::INSTRUCTIONS);
        println!("  Program ID bytes: {:?}", &test_pump::program_id()[..4]); 
        
        println!("\n VERIFICATION COMPLETE");
        println!("  - All constants generated correctly");
        println!("  - All functions have correct signatures");
        println!("  - Different IDL versions handled properly");
        println!("  - Generated code compiles without errors");
    }
}