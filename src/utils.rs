// pub(crate) fn clean_string(input: &[char]) -> &[char] {
//   for i in 0..input.len() {
//     if input[i] == '\0' {
//       return &input[0..i];
//     }
//   }
//
//   input
// }

const NULL_BYTE: char = '\0';

pub(crate) fn clean_string(input: &[char]) -> String {
  let mut output = String::new();

  for c in input {
    if c == &NULL_BYTE {
      return output;
    }

    output.push(*c)
  }

  output
}
