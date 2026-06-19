var global u32;
var global2 u64 = 10000;

fn foo() {
    return 12;
}

fn main() {
    var c string;
    var e [10]u16;
    var a u8 = 10; 
    var d *u8 = &a;
    var b = a + -5 * 2;
    for i = 0; i < 10; i = i + 1 {
        b = foo();
    }
    if a == 10 {
        return b;
    }

    return ~b;
}
