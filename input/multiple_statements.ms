var glob u32;
var glob2 u64 = 10000;

fn foo(a int, b string) int {
    return 12;
}

fn main() {
    var c string;
    var a u8 = 10;
    var b = a + -5 * 2;
    for i = 0; i < 10; i = i + 1 {
        b = foo(b, c);
    }
    if a == 10 {
        return b;
    }

    return ~b;
}
