const NULL_BYTE: char = '\0';
const BACKSLASH: char = '\\';
const SLASH: char = '/';

pub fn clean_path(input: &[char]) -> String {
  let mut output = String::new();

  for c in input {
    if c == &NULL_BYTE {
      return output;
    }

    if x == &BACKSLASH {
      output.push(SLASH);
    } else {
      output.push(*c)
    }
  }

  output
}
