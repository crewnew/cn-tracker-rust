fn main() {
    #[cfg(target_os = "windows")]
    {
        use captis::*;
        use timetrackrs::graphql::send_screenshots;

        let capturer = init_capturer().unwrap();

        let image = capturer.capture(0).unwrap();

        send_screenshots(&[image]).unwrap();
    }
}
