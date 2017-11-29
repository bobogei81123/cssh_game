macro_rules! impl_loggable {
    ($name:ty) => {
        impl Loggable for $name {
            fn logger(&self) -> &Logger { &self.common.logger }
        }
    }
}

macro_rules! impl_output_sender_lifetime {
    ($name:ty, $output:ty) => {
        impl<'a> OutputSender for &'a $name {
            type Output = $output;

            fn get_send_sink(&self) -> &WsSender {
                &self.common.ws_sender
            }
        }

        impl<'a> OutputSender for &'a mut $name {
            type Output = $output;

            fn get_send_sink(&self) -> &WsSender {
                &self.common.ws_sender
            }
        }
    }
}
//macro_rules! derive_output_sender {
    //($output_def:stmt) => {
    //}
//}
