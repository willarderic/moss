fn main() {
	var left u64 = 10;
	var right u64 = 20;
	var result u64;
	var bool_result bool;

	result = left + right;
	result = left * right;
	result = left - right;
	result = left / right;

	result = left + 5;
	result = left * 5;
	result = left - 5;
	result = left / 5;

	result = 5 + right;
	result = 5 * right;
	result = 5 - right;
	result = 5 / right;
	
	bool_result = left < right;
	bool_result = left > right;
	bool_result = left <= right;
	bool_result = left >= right;
	bool_result = left == right;
	bool_result = left != right;

	bool_result = left < 5;
	bool_result = left > 5;
	bool_result = left <= 5;
	bool_result = left >= 5;
	bool_result = left == 5;
	bool_result = left != 5;

	bool_result = 5 < right;
	bool_result = 5 > right;
	bool_result = 5 <= right;
	bool_result = 5 >= right;
	bool_result = 5 == right;
	bool_result = 5 != right;

	return bool_result;
}
