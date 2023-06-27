mod common;
use common::*;

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
    test_successfull_execution(source, "test_if");
}
