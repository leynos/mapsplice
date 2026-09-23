    match local {
        Some(mapped) => Some(mapped),
        None => {
            if source_is_fragment { cross_unique } else { None }
        }
    }
