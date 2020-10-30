
fn _ffib(n: usize) -> (usize, usize) {
    if n == 0 {
        (0,1)
    } else {
        let (a,b) = _ffib(n / 2);
        let c = a * (b * 2 - a);
        let d = a * a + b * b;
        if n % 2 == 0{
            (c,d)
        } else {
            (d, c + d)
        }
    }
}

pub fn ffib(n:usize) -> usize {
    if n == 0 {
        0 
    } else {
        let (_,b) = _ffib(n-1);
        b
    }
}
