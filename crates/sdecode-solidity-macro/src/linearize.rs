use quick_impl::quick_impl_all;

pub fn c3_linearize(inheritance_list: &[Vec<usize>]) -> Result<Vec<Vec<usize>>, usize> {
    let mut results = vec![ComputationState::None; inheritance_list.len()];
    for target in 0..inheritance_list.len() {
        c3_linearize_inner(inheritance_list, target, &mut results)?;
    }
    Ok(results
        .into_iter()
        .map(|x| x.try_into_finalized().unwrap())
        .collect())
}

#[derive(Debug, Clone)]
#[quick_impl_all(pub const is, pub set)]
enum ComputationState<T> {
    #[quick_impl(impl Default)]
    None,

    Pending,

    #[quick_impl(pub try_into)]
    Finalized(T),
}

fn c3_linearize_inner(
    inheritance_list: &[Vec<usize>],
    target: usize,
    results: &mut [ComputationState<Vec<usize>>],
) -> Result<(), usize> {
    match &mut results[target] {
        r @ ComputationState::None => {
            r.set_pending();
        }
        ComputationState::Pending => return Err(target),
        ComputationState::Finalized(_) => return Ok(()),
    }

    let parents = &inheritance_list[target];

    for parent in parents {
        c3_linearize_inner(inheritance_list, *parent, results)?;
    }

    let mut to_merge = Vec::with_capacity(parents.len());
    for parent in parents {
        let linearized = results[*parent]
            .clone()
            .try_into_finalized()
            .expect("linearization computed above");
        to_merge.push(linearized);
    }
    to_merge.push(parents.clone());

    let Some(mut result) = merge(to_merge, inheritance_list.len()) else {
        return Err(target);
    };
    result.insert(0, target);

    results[target] = ComputationState::Finalized(result);
    Ok(())
}

fn merge<T: PartialEq + Clone>(mut input: Vec<Vec<T>>, capacity: usize) -> Option<Vec<T>> {
    let mut result = Vec::with_capacity(capacity);

    input.retain(|inner| !inner.is_empty());

    while !input.is_empty() {
        let head = find_good_head(&input)?;

        input.retain_mut(|inner| {
            inner.retain(|item| *item != head);
            !inner.is_empty()
        });

        result.push(head);
    }

    Some(result)
}

fn find_good_head<T: PartialEq + Clone>(input: &[Vec<T>]) -> Option<T> {
    for inner in input {
        let head = inner.first().unwrap();
        if is_good_head(input, head) {
            return Some(head.clone());
        }
    }
    None
}

fn is_good_head<T: PartialEq>(input: &[Vec<T>], head: &T) -> bool {
    for other_input in input {
        if tail(other_input).contains(head) {
            return false;
        }
    }
    true
}

fn tail<T>(input: &[T]) -> &[T] {
    if input.is_empty() { input } else { &input[1..] }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Examples from <https://en.wikipedia.org/wiki/C3_linearization#Example>

    #[test]
    fn test_c3_lin_merge() {
        let merged = merge(vec![vec!["O"]], 10).unwrap();
        assert_eq!(merged, vec!["O"], "O");

        let merged = merge(vec![vec!["O"], vec!["O"]], 10).unwrap();
        assert_eq!(merged, vec!["O"], "A");

        let merged = merge(
            vec![
                vec!["C", "O"],
                vec!["B", "O"],
                vec!["A", "O"],
                vec!["C", "A", "B"],
            ],
            10,
        )
        .unwrap();
        assert_eq!(merged, vec!["C", "A", "B", "O"], "K1");

        let merged = merge(vec![vec!["A", "O"], vec!["D", "O"], vec!["A", "D"]], 10).unwrap();
        assert_eq!(merged, vec!["A", "D", "O"], "K3");

        let merged = merge(
            vec![
                vec!["B", "O"],
                vec!["D", "O"],
                vec!["E", "O"],
                vec!["B", "D", "E"],
            ],
            10,
        )
        .unwrap();
        assert_eq!(merged, vec!["B", "D", "E", "O"], "K2");

        let merged = merge(
            vec![
                vec!["K1", "C", "A", "B", "O"],
                vec!["K3", "A", "D", "O"],
                vec!["K2", "B", "D", "E", "O"],
                vec!["K1", "K3", "K2"],
            ],
            10,
        )
        .unwrap();
        assert_eq!(
            merged,
            vec!["K1", "C", "K3", "A", "K2", "B", "D", "E", "O"],
            "Z"
        );
    }

    #[test]
    fn test_c3_linearize() {
        struct Indexes {
            o: usize,
            a: usize,
            b: usize,
            c: usize,
            d: usize,
            e: usize,
            k1: usize,
            k3: usize,
            k2: usize,
            z: usize,
        }
        let ids = Indexes {
            o: 0,
            a: 1,
            b: 2,
            c: 3,
            d: 4,
            e: 5,
            k1: 6,
            k3: 7,
            k2: 8,
            z: 9,
        };

        let lin = c3_linearize(&[
            vec![],
            vec![ids.o],
            vec![ids.o],
            vec![ids.o],
            vec![ids.o],
            vec![ids.o],
            vec![ids.c, ids.a, ids.b],
            vec![ids.a, ids.d],
            vec![ids.b, ids.d, ids.e],
            vec![ids.k1, ids.k3, ids.k2],
        ])
        .unwrap();

        assert_eq!(&lin[ids.o], &[ids.o], "O");
        assert_eq!(&lin[ids.a], &[ids.a, ids.o], "A");
        assert_eq!(&lin[ids.b], &[ids.b, ids.o], "B");
        assert_eq!(&lin[ids.c], &[ids.c, ids.o], "C");
        assert_eq!(&lin[ids.d], &[ids.d, ids.o], "D");
        assert_eq!(&lin[ids.e], &[ids.e, ids.o], "E");
        assert_eq!(&lin[ids.k1], &[ids.k1, ids.c, ids.a, ids.b, ids.o], "K1");
        assert_eq!(&lin[ids.k3], &[ids.k3, ids.a, ids.d, ids.o], "K3");
        assert_eq!(&lin[ids.k2], &[ids.k2, ids.b, ids.d, ids.e, ids.o], "K2");
        assert_eq!(
            &lin[ids.z],
            &[
                ids.z, ids.k1, ids.c, ids.k3, ids.a, ids.k2, ids.b, ids.d, ids.e, ids.o
            ],
            "Z"
        );
    }
}
