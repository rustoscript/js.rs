function f() {
    var x = 0;

    return function() {
	return x++;
    };
}

var g = f();
var h = f();

log("Counter 1:");
log(g());
log(g());
log(g());

log("Counter 2:");
log(h());
log(h());
log(h());
