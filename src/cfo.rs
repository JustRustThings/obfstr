/*!
Control Flow Obfuscation
========================
*/

/// Generates the keys and xor values for a sequence of statements.
pub const fn generate<const LEN: usize>(mut key: u32, mut xor: u32, stmts: &[&'static str; LEN]) -> [(&'static str, u32, u32); LEN] {
	let mut result = [("", 0, 0); LEN];
	let mut i = 0;
	while i < stmts.len() {
		key ^= xor;
		xor = crate::murmur3(stmts[i].as_bytes(), key);
		// FIXME! This should check for collisions...
		result[i] = (stmts[i], key, xor);
		i += 1;
	}
	result
}

/// Statement control flow obfuscation.
///
/// Given a sequence of statements obfuscates the relationship between each statement.
///
/// # Limitations
///
/// Variables cannot be declared inside the obfuscated statements, declare and initialize any variables needed beforehand.
/// Control flow analysis will fail. The declared variables will need to be mutable and have an initial value.
///
/// There's a risk that the obfuscated code fails to work due to two statements generating the same random key accidentally.
/// This is presented at runtime with an infinite loop pending extra validation checks.
///
/// # Examples
///
/// ```
/// let mut tmp = 0;
/// obfstr::obfstmt! {
/// 	tmp = 2;
/// 	tmp *= 22;
/// 	tmp -= 12;
/// 	tmp /= 3;
/// }
///# obfstr::obfstmt! {}
/// assert_eq!(tmp, 10);
/// ```
#[macro_export]
macro_rules! obfstmt {
	($($stmt:stmt;)*) => {{
		// Initial KEY and XOR values
		const _OBFSTMT_KEY: u32 = $crate::random!(u32);
		const _OBFSTMT_XOR: u32 = $crate::murmur3(b"XOR", _OBFSTMT_KEY);
		// Count the number of statements
		const _OBFSTMT_LEN: usize = <[&'static str]>::len(&[$(stringify!($stmt)),*]);
		// Generate key and xor values of every statement and the final exit code
		const _OBFSTMT_STMTS: [(&'static str, u32, u32); _OBFSTMT_LEN] =
			$crate::cfo::generate::<{_OBFSTMT_LEN}>(_OBFSTMT_KEY, _OBFSTMT_XOR, &[$(stringify!($stmt)),*]);
		const _OBFSTMT_EXIT: u32 = if _OBFSTMT_LEN == 0 { _OBFSTMT_KEY ^ _OBFSTMT_XOR }
			else { _OBFSTMT_STMTS[_OBFSTMT_LEN - 1].1 ^ _OBFSTMT_STMTS[_OBFSTMT_LEN - 1].2 };
		// Initialize the key and xor values
		let mut key = _OBFSTMT_KEY;
		#[allow(unused_mut)]
		let mut xor = _OBFSTMT_XOR;
		loop {
			$crate::__obfstmt_match!(key, xor, 0usize, [$($stmt;)*], []);
			key ^= xor;
		}
	}};
}

/// Generates the match statement for [`obfstmt!`].
#[doc(hidden)]
#[macro_export]
macro_rules! __obfstmt_match {
	// Terminating case, generate the code
	($key:expr, $xor:expr, $x:expr, [], [$($i:expr, $stmt:stmt;)*]) => {
		match $key {
			// Have to use match guard here because an expression isn't allowed in pattern position
			// The result is still optimized to a binary search for the right key per block
			$(
				key if key == { _OBFSTMT_STMTS[$i].1 } => {
					$stmt
					$xor = _OBFSTMT_STMTS[$i].2;
				},
			)*
			_OBFSTMT_EXIT => break,
			_ => (),
		}
	};
	// Generate increasing indices for every stmt
	($key:expr, $xor:expr, $x:expr, [$stmt1:stmt; $($tail:stmt;)*], [$($i:expr, $stmt2:stmt;)*]) => {
		$crate::__obfstmt_match!($key, $xor, $x + 1usize, [$($tail;)*], [$($i, $stmt2;)* $x, $stmt1; ])
	};
}

#[test]
fn test_identical_stmt() {
	let mut i: u8 = 0;
	obfstmt! {
		i += 1;
		i += 1;
		i += 1;
		i += 1;
	}
	obfstmt! {}
	assert_eq!(i, 4);
}
