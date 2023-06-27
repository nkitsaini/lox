#[macro_use]
mod common;
// use common::*;

#[test]
fn test_if() {
    let source = "

    print 0;
	if ( 1 < 3) {
		print 1;
	}
    if (1 > 3) {
        print 2;
    }
    print 3;
";

    test_execution_success!(source);
}
#[test]
fn test_if_else() {
    let source = r#"

    print 0;
	if ( 1 > 3) {
		print 1;
	}
    else  {
        print 2;
    }

	if ( 1 < 3) {
		print 3;
	}
    else  {
        print 4;
    }
    
    print 5;
"#;

    test_execution_success!(source);
}
