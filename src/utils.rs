pub(crate) fn clean_string(input: &[char]) -> &[char] {
  for i in 0..input.len() {
    if input[i] == '\0' {
      return &input[0..i];
    }
  }

  input
}
