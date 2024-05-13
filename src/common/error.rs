use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(visibility(pub), display("listen {addr} fail!"))]
    Net {
        source: std::io::Error,
        addr: String,
    },

    #[snafu(visibility(pub), display("read fail"))]
    Read { source: std::io::Error },

    #[snafu(visibility(pub), display("write fail"))]
    Write { source: std::io::Error },

    #[snafu(visibility(pub), display("frame_writer send fail : {msg}"))]
    Frame { msg: String },

    #[snafu(visibility(pub), display("channel error : {msg}"))]
    Channel { msg: String },

    #[snafu(visibility(pub), display("group error : {msg}"))]
    Group { msg: String },
}
