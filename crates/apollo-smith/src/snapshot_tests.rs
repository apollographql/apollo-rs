use crate::DocumentBuilder;
use arbitrary::Unstructured;
use expect_test::expect;

fn gen(len: usize) -> String {
    let entropy: Vec<u8> = (0..len).map(|i| i as u8).collect();
    DocumentBuilder::new(&mut Unstructured::new(&entropy))
        .unwrap()
        .finish()
        .into()
}

#[test]
fn snapshot_tests() {
    expect![[r#"
        {
          A0
        }

        fragment A2 on A1 {
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
        }

        fragment A7 on A6 {
          A0
        }

        schema {
          query: A6
          mutation: A6
          subscription: A6
        }

        scalar CD

        type A6 {
          A0: IJAAAAAA
          A1: IJAAAAAA
        }

        interface A6 {
          A0: IJAAAAAA
          A1: IJAAAAAA
        }

        union A7 = A6

        enum IJAAAAAA {
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

        input A7 {
          A0: A6
          A1: A6
        }

        directive @A7 on QUERY
    "#]]
    .assert_eq(&gen(10));
    expect![[r#"
        {
          A0
        }

        fragment A7 on A6 {
          A0
        }

        schema {
          query: A6
          mutation: A6
          subscription: A6
        }

        scalar CD

        type A6 {
          A0: IJKLMNOP
          A1: IJKLMNOP
        }

        interface A6 {
          A0: IJKLMNOP
          A1: IJKLMNOP
        }

        union A7 = A6

        enum IJKLMNOP {
          UVWXYZabcdefghijklmn0
          qrstuvwxyz_01
          E456789ABCDEFGHIJKLMNOPQRS2
          gWXYZabcdefghijkAAAAAAAA3
          A4
          A5
          A6
          A7
          A8
          A9
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

        input A7 {
          A0: A6
          A1: A6
        }

        directive @A7 on QUERY
    "#]]
    .assert_eq(&gen(100));
    expect![[r#"
        {
          A0
        }

        fragment A7 on A6 {
          A0
        }

        schema {
          query: A6
          mutation: A6
          subscription: A6
        }

        scalar CD

        type A6 {
          A0: IJKLMNOP
          A1: IJKLMNOP
        }

        "9ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_\n\r	"
        interface Jdefghi {
          SmnopqAAAAAAAAAA0: IJKLMNOP
          A1: IJKLMNOP
          A2: IJKLMNOP
          A3: IJKLMNOP
          A4: IJKLMNOP
          A5: IJKLMNOP
          A6: IJKLMNOP
          A7: IJKLMNOP
          A8: IJKLMNOP
          A9: IJKLMNOP
          A10: IJKLMNOP
          A11: IJKLMNOP
          A12: IJKLMNOP
          A13: IJKLMNOP
          A14: IJKLMNOP
          A15: IJKLMNOP
          A16: IJKLMNOP
          A17: IJKLMNOP
          A18: IJKLMNOP
          A19: IJKLMNOP
          A20: IJKLMNOP
          A21: IJKLMNOP
          A22: IJKLMNOP
          A23: IJKLMNOP
          A24: IJKLMNOP
          A25: IJKLMNOP
          A26: IJKLMNOP
          A27: IJKLMNOP
          A28: IJKLMNOP
          A29: IJKLMNOP
        }

        interface A {
          A0: IJKLMNOP
          A1: IJKLMNOP
        }

        interface A6 {
          A0: IJKLMNOP
          A1: IJKLMNOP
        }

        interface A6 {
          A0: IJKLMNOP
          A1: IJKLMNOP
        }

        interface A6 {
          A0: IJKLMNOP
          A1: IJKLMNOP
        }

        interface A6 {
          A0: IJKLMNOP
          A1: IJKLMNOP
        }

        interface A6 {
          A0: IJKLMNOP
          A1: IJKLMNOP
        }

        union A7 = A6

        enum IJKLMNOP {
          UVWXYZabcdefghijklmn0
          qrstuvwxyz_01
          E456789ABCDEFGHIJKLMNOPQRS2
          gWXYZabcdefghijklmnopqrs3
          Gwxyz_0123456789ABCD4
          cHIJKLMNOPQR5
          qVWXYZabcdefghijklmnopqrst6
          Sxyz_0123456789ABCDEFGHI7
          sMNOPQRSTUVWXYZabcde8
          Oijklmnopqrs9
        }

        """CDEFGHIJKLMNOPQRSTU0123456789ABCDE"""
        enum QRSTUVWXYZabcdef {
          klmnop0
          stuvwxyz_012341
          I89ABCDEFGHIJKLMNOPQRSTUVWXYZa2
          oe3
          sijklm4
          Aqrstuvwxyz_015
          Q56789ABCDEFGHIJKLMNOPQRSTUVWX6
          wb7
        }

        """789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"""
        enum FZa {
          Mghijklmno0
          Ystuvwxyz_0123456789AB1
          ABCDEFGHIJKLMNOP2
        }

        """JKLMNOPQRSTUVWXYZa"""
        enum mnopqrst {
          yz_0123456789ABCDEFG0
          UKLMNOPQRSTU1
          iYZabcdefghijklmnopqrstuvw2
          K_0123456789ABCDEFGHIJKL3
        }

        """/$#!.-+='0123456789ABCDEFGHIJKLMNOPQRST"""
        enum Z {
          e89ABC0
          mGHIJKLMNOPQRS1
          CWXYZabcdefghijklmnopqrstuvwxy2
          i13
          m567894
        }

        """U0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmno"""
        enum A0123456789ABCDEFGHIJK {
          aQRSTUVWXYZabcdefg0
          uklmnopq1
          Euvwxyz_01234567892
          YDEFGHIJ3
          iNOPQRSTUVWXYZabcd4
        }

        input A7 {
          A0: A6
          A1: A6
        }

        directive @A7 on QUERY
    "#]]
    .assert_eq(&gen(1000));
}
