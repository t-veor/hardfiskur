use std::fmt::Debug;

use pretty_assertions::assert_eq;

pub fn assert_in_any_order<T: Eq + Ord + Debug>(
    values: impl IntoIterator<Item = T>,
    expected: impl IntoIterator<Item = T>,
) {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();

    let mut expected = expected.into_iter().collect::<Vec<_>>();
    expected.sort();

    assert_eq!(values, expected);
}
