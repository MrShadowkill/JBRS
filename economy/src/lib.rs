pub fn purchase(balance: &mut u32, owned: &mut Vec<usize>, kind: usize, price: u32) -> bool {
    if owned.contains(&kind) {
        return true;
    }
    if *balance < price {
        return false;
    }
    *balance -= price;
    owned.push(kind);
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cannot_overspend() {
        let mut b = 0;
        let mut v = vec![0, 1];
        assert!(!purchase(&mut b, &mut v, 3, 2500));
        assert_eq!(b, 0);
    }
}
