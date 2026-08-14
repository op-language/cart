use cart::opc;
use std::process::Command;

#[test]
fn output_extension_mapping() {
    assert_eq!(opc::output_extension("ines"), "nes");
    assert_eq!(opc::output_extension("lnx"), "lnx");
    assert_eq!(opc::output_extension("raw"), "bin");
    assert_eq!(opc::output_extension("hex"), "hex");
    assert_eq!(opc::output_extension("unknown"), "bin");
}

#[test]
fn opc_args_build_command() {
    use std::path::PathBuf;
    let args = opc::OpcArgs {
        input: PathBuf::from("game.op"),
        target: "mos6502-nintendo-nes-ntsc".to_string(),
        features: vec!["debug".to_string()],
        opt_level: 1,
        format: Some("ines".to_string()),
        output: Some(PathBuf::from("out.nes")),
        stage: opc::OpcStage::Full,
    };
    let cmd = args.to_command();
    let program = cmd.get_program().to_string_lossy();
    assert_eq!(program, "opc");
}

#[test]
fn opc_args_parse_stage() {
    use std::path::PathBuf;
    let args = opc::OpcArgs {
        input: PathBuf::from("game.op"),
        target: "mos6502-nintendo-nes-ntsc".to_string(),
        features: Vec::new(),
        opt_level: 0,
        format: None,
        output: None,
        stage: opc::OpcStage::Parse,
    };
    let cmd = args.to_command();
    let cmd_args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();
    assert!(cmd_args.contains(&"--parse".to_string()));
}
