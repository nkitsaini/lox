// These are macros otherwise insta snapshot goes crazy.
// It starts having different folder locations for `cargo insta test` and `cargo test`

macro_rules! test_execution {
    ($result:expr, $script:expr) => {{
        let mut stdout = std::io::Cursor::new(Vec::new());
        let mut stderr = std::io::Cursor::new(Vec::new());
        let mut vm = rlox::vm::VM::empty_new(&mut stdout, &mut stderr);
        let res = vm.interpret($script);

        drop(vm);

        let stderr = String::from_utf8(stderr.into_inner()).unwrap();
        let stdout = String::from_utf8(stdout.into_inner()).unwrap();
        println!("======== stdout: \n{}", &stdout);
        println!("======== stderr: \n{}", &stderr);
        pretty_assertions::assert_eq!(res, $result);
        insta::assert_snapshot!(stdout);
        insta::assert_snapshot!(stderr);
    }};
}
macro_rules! test_execution_success {
    ($script:ident) => {
        test_execution!(Ok(()), $script);
    };
}
