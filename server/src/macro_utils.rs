macro_rules! capture {
    ($($n: ident),* => $body:expr) => (
        {
            $( let $n = $n.clone(); )*
            $body
        }
    );
    //($($n: ident),* => |$($p:tt : $typ: typ),*| $body:expr) => (
        //{
            //$( let $n = $n.clone(); )*
            //move |$($p: $typ),*| $body
        //}
    //);
}

macro_rules! consume_result {
    ($fut: expr) => {
        $fut.map_err(|_| ()).map(|_| ())
    };
    ($fut: expr, $map_err: expr) => {
        $fut.map_err($map_err).map(|_| ())
    };
    ($fut: expr, $map_err: expr, $map: expr) => {
        $fut.map_err($map_err).map($map)
    }
}


macro_rules! ensure {
    ($e: expr) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    }
}
