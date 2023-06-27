use stream_fusion::prelude::*;

fn main() {
    spin_on::spin_on(async {
        let n = (0..2048)
            .into_fusion()
            .step_by(2)
            .map(|i| i + 1)
            .filter(|i| *i > 512)
            .fold(0, |acc, item| acc + item)
            .yield_by(32)
            .await;

        let expect = (0..2048)
            .step_by(2)
            .map(|i| i + 1)
            .filter(|i| *i > 512)
            .fold(0, |acc, item| acc + item);

        assert_eq!(n, expect);
    })
}
