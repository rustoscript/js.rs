function factorial(i) {
    if (i < 2) {
	return 1;
    }

    return i * factorial(i - 1);
}

log("1! = " + factorial(1));
log("5! = " + factorial(5));
log("11! = " + factorial(11));
