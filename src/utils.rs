const NULL_BYTE: char = '\0';
const BACKSLASH: char = '\\';
const SLASH: char = '/';

pub(crate) fn clean_path(input: &[char]) -> String {
  let mut output = String::new();

  for c in input {
    if c == &NULL_BYTE {
      return output;
    }

    if c == &BACKSLASH {
      output.push(SLASH);
    } else {
      output.push(*c)
    }
  }

  output
}
