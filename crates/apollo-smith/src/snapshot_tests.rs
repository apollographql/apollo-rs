use crate::DocumentBuilder;
use arbitrary::Unstructured;
use expect_test::expect;

fn gen(len: usize) -> String {
    let entropy: Vec<u8> = (0..len).map(|i| i as u8).collect();
    DocumentBuilder::new(&mut Unstructured::new(&entropy), false)
        .unwrap()
        .finish()
        .into()
}

#[test]
fn snapshot_tests() {
    expect![[r#"
        {
          A0
          A0
        }

        fragment A2 on A1 {
          A0
          A0
        }

        schema {
          query: A1
          mutation: A1
          subscription: A1
        }

        scalar A

        type A1 {
          A0: A0
          A1: A0
        }

        interface A1 {
          A0: A0
          A1: A0
        }

        union A2 = A1

        enum A0 {
          A0
          A1
        }

        input A2 {
          A0: A1
          A1: A1
        }

        directive @A2 on QUERY
    "#]]
    .assert_eq(&gen(0));
    expect![[r#"
        {
          A0
          A0
        }

        fragment A2 on A1 {
          A0
          A0
        }

        schema {
          query: A1
          mutation: A1
          subscription: A1
        }

        scalar CA

        type A1 {
          A0: A
          A1: A
        }

        interface A1 {
          A0: A
          A1: A
        }

        union A2 = A1

        enum A {
          A0
          A1
        }

        input A2 {
          A0: A1
          A1: A1
        }

        directive @A2 on QUERY
    "#]]
    .assert_eq(&gen(10));
    expect![[r#"
        {
          A0
          A0
        }

        fragment A21 on A20 {
          A0
          A0
        }

        schema {
          query: A20
          mutation: A20
          subscription: A20
        }

        scalar CJ

        type A20 {
          A0: uECA86420zAAAAAAAAAAAA
          A1: uECA86420zAAAAAAAAAAAA
        }

        interface A20 {
          A0: uECA86420zAAAAAAAAAAAA
          A1: uECA86420zAAAAAAAAAAAA
        }

        union A21 = A20

        enum uECA86420zAAAAAAAAAAAA {
          A0
          A1
        }

        enum A {
          A0
          A1
        }

        enum A2 {
          A0
          A1
        }

        enum A3 {
          A0
          A1
        }

        enum A4 {
          A0
          A1
        }

        enum A5 {
          A0
          A1
        }

        enum A6 {
          A0
          A1
        }

        enum A7 {
          A0
          A1
        }

        enum A8 {
          A0
          A1
        }

        enum A9 {
          A0
          A1
        }

        enum A10 {
          A0
          A1
        }

        enum A11 {
          A0
          A1
        }

        enum A12 {
          A0
          A1
        }

        enum A13 {
          A0
          A1
        }

        enum A14 {
          A0
          A1
        }

        enum A15 {
          A0
          A1
        }

        enum A16 {
          A0
          A1
        }

        enum A17 {
          A0
          A1
        }

        enum A18 {
          A0
          A1
        }

        enum A19 {
          A0
          A1
        }

        input A21 {
          A0: A20
          A1: A20
        }

        directive @A21 on QUERY
    "#]]
    .assert_eq(&gen(100));
    expect![[r#"
        {
          A0
          A0
        }

        fragment A21 on A20 {
          A0
          A0
        }

        schema {
          query: A20
          mutation: A20
          subscription: A20
        }

        scalar CJ

        type A20 {
          A0: uECA86420zxvtrpnljhfdb
          A1: uECA86420zxvtrpnljhfdb
        }

        interface A20 {
          A0: uECA86420zxvtrpnljhfdb
          A1: uECA86420zxvtrpnljhfdb
        }

        union A21 = A20

        enum uECA86420zxvtrpnljhfdb {
          aWUSQO2LJHFDB97531_ywu0
          AKIG1
          ChfdbZXV2
        }

        "S_LdpAi3b-U\rNvGo9h2a.T"
        enum mkigecaYWUSQO2LJHFDB97531 {
          Ovtrpnljhfdb0
          g1_yAAAAAAAAAAAAAAAA1
        }

        enum A {
          A0
          A1
        }

        enum A3 {
          A0
          A1
        }

        enum A4 {
          A0
          A1
        }

        enum A5 {
          A0
          A1
        }

        enum A6 {
          A0
          A1
        }

        enum A7 {
          A0
          A1
        }

        enum A8 {
          A0
          A1
        }

        enum A9 {
          A0
          A1
        }

        enum A10 {
          A0
          A1
        }

        enum A11 {
          A0
          A1
        }

        enum A12 {
          A0
          A1
        }

        enum A13 {
          A0
          A1
        }

        enum A14 {
          A0
          A1
        }

        enum A15 {
          A0
          A1
        }

        enum A16 {
          A0
          A1
        }

        enum A17 {
          A0
          A1
        }

        enum A18 {
          A0
          A1
        }

        enum A19 {
          A0
          A1
        }

        input A21 {
          A0: A20
          A1: A20
        }

        directive @A21 on QUERY
    "#]]
    .assert_eq(&gen(1000));
}
