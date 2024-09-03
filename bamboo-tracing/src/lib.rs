// fn setup_tracing() -> Result<(), error::Error> {
//     let file_appender = tracing_appender::rolling::daily("./tracing", "quant.log");
//     let (file_non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
//     let file_layer = tracing_subscriber::fmt::layer()
//         .json()
//         .with_line_number(true)
//         .with_writer(file_non_blocking)
//         .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::fmt::layer()
//                 .pretty()
//                 .with_line_number(true)
//                 .with_writer(std::io::stderr)
//                 .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
//         )
//         .with(file_layer)
//         .init();

//     Ok(())
// }

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
