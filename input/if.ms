fn main() {
	var x u64 = 20;
	var y u64 = 10;
	if x > y {
		return x - y;
	} else if x < y {
		return y - x;
	} else {
		return 0;
	}
}
