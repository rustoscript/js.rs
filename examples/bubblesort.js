function bubblesort(array) {
    var changed = true;

    while (changed) {
	changed = false;
	var i = 0;

	while (i < array.length - 1) {
	    if (array[i] > array[i + 1]) {
		var temp = array[i];
		array[i] = array[i + 1];
		array[i + 1] = temp;
		changed = true;
	    }

	    i += 1;
	}
    }
}

var a = [3,5,7,9,1,2,4,8,0];

log("Original array:");
log(a);

bubblesort(a);

log("\nSorted array:");
log(a);
