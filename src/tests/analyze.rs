use analyze_macro::analyze;

#[test]
fn analyze() {
    analyze!(
        /// outer comment
        /** block comment */
        struct Person {
            /// field comment
            /** inner block comment */
            name: String,
            /// field comment
            age: u32,
        }
    )
}
