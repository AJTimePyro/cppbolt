// Formatting helpers for the C printf runtime contract.
// These reproduce the exact C printf rounding semantics for float/double output.

function roundToEvenExact(vNum: number, prec: number): string {
	if (Number.isNaN(vNum)) return "nan";
	if (vNum === Infinity) return "inf";
	if (vNum === -Infinity) return "-inf";
	if (Object.is(vNum, -0)) {
		return `-0${prec > 0 ? `.${"0".repeat(prec)}` : ""}`;
	}
	if (vNum === 0) {
		return `0${prec > 0 ? `.${"0".repeat(prec)}` : ""}`;
	}
	const isNeg = vNum < 0;
	const v = Math.abs(vNum);

	const buf = new ArrayBuffer(8);
	const view = new DataView(buf);
	view.setFloat64(0, v, false);
	const bits = view.getBigUint64(0, false);
	const rawExp = Number((bits >> 52n) & 0x7ffn);
	const rawMantissa = bits & 0xfffffffffffffn;

	let mantissa: bigint;
	let exp: number;
	if (rawExp === 0) {
		mantissa = rawMantissa;
		exp = -1022 - 52;
	} else {
		mantissa = rawMantissa | (1n << 52n);
		exp = rawExp - 1023 - 52;
	}

	const P = BigInt(prec);
	const p5 = 5n ** P;
	const expP = BigInt(exp) + P;

	let num: bigint;
	let den: bigint;
	if (expP >= 0n) {
		num = mantissa * 2n ** expP * p5;
		den = 1n;
	} else {
		num = mantissa * p5;
		den = 2n ** -expP;
	}

	const Q = num / den;
	const R = num % den;
	const twoR = R * 2n;

	let rounded: bigint;
	if (twoR < den) {
		rounded = Q;
	} else if (twoR > den) {
		rounded = Q + 1n;
	} else {
		rounded = Q % 2n === 0n ? Q : Q + 1n;
	}

	let s = rounded.toString();
	if (prec > 0) {
		while (s.length <= prec) {
			s = `0${s}`;
		}
		const intPart = s.slice(0, s.length - prec);
		const fracPart = s.slice(s.length - prec);
		s = `${intPart}.${fracPart}`;
	}
	return (isNeg ? "-" : "") + s;
}

function formatExpExact(vNum: number, prec: number): string {
	if (Number.isNaN(vNum)) return "nan";
	if (vNum === Infinity) return "inf";
	if (vNum === -Infinity) return "-inf";
	if (Object.is(vNum, -0)) {
		const zeros = prec > 0 ? `.${"0".repeat(prec)}` : "";
		return `-0${zeros}e+00`;
	}
	if (vNum === 0) {
		const zeros = prec > 0 ? `.${"0".repeat(prec)}` : "";
		return `0${zeros}e+00`;
	}
	const isNeg = vNum < 0;
	const v = Math.abs(vNum);
	let exp = Math.floor(Math.log10(v));

	const buf = new ArrayBuffer(8);
	const view = new DataView(buf);
	view.setFloat64(0, v, false);
	const bits = view.getBigUint64(0, false);
	const rawExp = Number((bits >> 52n) & 0x7ffn);
	const rawMantissa = bits & 0xfffffffffffffn;

	let mantissa: bigint;
	let bexp: number;
	if (rawExp === 0) {
		mantissa = rawMantissa;
		bexp = -1022 - 52;
	} else {
		mantissa = rawMantissa | (1n << 52n);
		bexp = rawExp - 1023 - 52;
	}

	let Q = 0n;
	let R = 0n;
	let den = 1n;
	while (true) {
		const shift = prec - exp;
		const S = BigInt(shift);
		let num: bigint;
		if (S >= 0n) {
			const p5 = 5n ** S;
			const expP = BigInt(bexp) + S;
			if (expP >= 0n) {
				num = mantissa * 2n ** expP * p5;
				den = 1n;
			} else {
				num = mantissa * p5;
				den = 2n ** -expP;
			}
		} else {
			const negS = -S;
			const p5 = 5n ** negS;
			const expP = BigInt(bexp) - negS;
			if (expP >= 0n) {
				num = mantissa * 2n ** expP;
				den = p5;
			} else {
				num = mantissa;
				den = p5 * 2n ** -expP;
			}
		}
		Q = num / den;
		R = num % den;
		if (Q === 0n && v > 0) {
			exp--;
			continue;
		}
		if (Q >= 10n ** BigInt(prec + 1)) {
			exp++;
			continue;
		}
		break;
	}

	const twoR = R * 2n;
	let rounded: bigint;
	if (twoR < den) {
		rounded = Q;
	} else if (twoR > den) {
		rounded = Q + 1n;
	} else {
		rounded = Q % 2n === 0n ? Q : Q + 1n;
	}

	if (rounded >= 10n ** BigInt(prec + 1)) {
		rounded = rounded / 10n;
		exp++;
	}

	let s = rounded.toString();
	while (s.length < prec + 1) {
		s = `0${s}`;
	}
	const mStr = s[0] + (prec > 0 ? `.${s.slice(1)}` : "");
	const expSign = exp >= 0 ? "+" : "-";
	const expVal = Math.abs(exp).toString().padStart(2, "0");
	return `${(isNeg ? "-" : "") + mStr}e${expSign}${expVal}`;
}

function formatGExact(vNum: number, prec: number, alternate = false): string {
	if (Number.isNaN(vNum)) return "nan";
	if (vNum === Infinity) return "inf";
	if (vNum === -Infinity) return "-inf";
	if (!prec || prec <= 0) prec = 1;
	if (Object.is(vNum, -0) || vNum === 0) {
		if (!alternate) return Object.is(vNum, -0) ? "-0" : "0";
		const res = roundToEvenExact(vNum, prec - 1);
		return res.includes(".") ? res : `${res}.`;
	}
	const v = Math.abs(vNum);
	const exp = Math.floor(Math.log10(v));
	let res: string;
	if (exp < -4 || exp >= prec) {
		res = formatExpExact(vNum, prec - 1);
		if (!alternate) {
			let [m, e] = res.split("e");
			if (m.includes(".")) {
				m = m.replace(/0+$/, "").replace(/\.$/, "");
			}
			res = `${m}e${e}`;
		} else if (Number.isFinite(vNum) && !res.includes(".")) {
			const e = res.indexOf("e");
			res = e >= 0 ? `${res.slice(0, e)}.${res.slice(e)}` : `${res}.`;
		}
	} else {
		const pAfterDot = Math.max(0, prec - 1 - exp);
		res = roundToEvenExact(vNum, pAfterDot);
		if (!alternate) {
			if (res.includes(".")) {
				res = res.replace(/0+$/, "").replace(/\.$/, "");
			}
		} else if (Number.isFinite(vNum) && !res.includes(".")) {
			res += ".";
		}
	}
	return res;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Options for the runtime import environment used by compiled programs.
 */
export interface WasmHostOptions {
	/**
	 * Stdin source for `read_char`.
	 * - A `string` is consumed character by character; EOF returns -1.
	 * - A `() => number` callback is called once per character; return -1 for EOF.
	 * - Omit (or leave undefined) for no stdin input.
	 */
	stdin?: string | (() => number);

	/**
	 * Called whenever the program writes characters to stdout.
	 * Receives the chunk as a string; chunks are flushed per print call,
	 * not per character.
	 */
	onStdout?: (chunk: string) => void;

	/**
	 * Called when the program invokes `exit(code)`.
	 * If omitted, falls back to `process.exit(code)` in Node.js. Browser hosts
	 * should provide this hook because there is no `process` to terminate.
	 */
	onExit?: (code: number) => void;
}

/**
 * The runtime host for a compiled C++ WebAssembly program.
 * Pass `imports` to `WebAssembly.instantiate`, then call `getStdout()` to
 * retrieve the accumulated output after the program finishes.
 */
export interface WasmHost {
	/** The WebAssembly import object — provide this to `WebAssembly.instantiate`. */
	imports: WebAssembly.Imports;
	/** Returns all stdout output accumulated since the host was created. */
	getStdout(): string;
}

/**
 * Creates the runtime import environment (`env.*`) required by programs
 * compiled with cppbolt.
 *
 * Every module emitted by the compiler imports exactly five functions:
 * `print_char`, `print_int`, `print_double`, `read_char`, and `exit`.
 * This function provides all five with correct printf-exact semantics that
 * match the compiler's runtime contract.
 */
export function createWasmHost(options: WasmHostOptions = {}): WasmHost {
	const { stdin, onStdout, onExit } = options;
	let stdoutBuffer = "";
	let inPos = 0;

	const writeStdout = (str: string): void => {
		stdoutBuffer += str;
		if (onStdout) {
			onStdout(str);
		}
	};

	const imports: WebAssembly.Imports = {
		env: {
			print_char: (ch: number): void => {
				writeStdout(String.fromCharCode(ch));
			},

			print_int: (
				val: bigint,
				radix: number,
				width: number,
				precision: number,
				flags: number,
			): number => {
				const isZeroPad = (flags & 1) !== 0;
				const isLeftAlign = (flags & 2) !== 0;
				const isShowSign = (flags & 4) !== 0;
				const isSpaceSign = (flags & 8) !== 0;
				const isUppercase = (flags & 16) !== 0;
				const isUnsigned = (flags & 32) !== 0;
				const isAlternate = (flags & 64) !== 0;

				let numVal = BigInt(val);
				let isNeg = false;
				if (isUnsigned) {
					numVal = BigInt.asUintN(64, numVal);
				} else {
					if (numVal < 0n) {
						isNeg = true;
						numVal = -numVal;
					}
				}

				let s = numVal.toString(radix);
				if (isUppercase) {
					s = s.toUpperCase();
				}

				if (precision >= 0) {
					if (precision === 0 && numVal === 0n) {
						s = "";
					} else {
						while (s.length < precision) {
							s = `0${s}`;
						}
					}
				}

				let signStr = "";
				if (isNeg) {
					signStr = "-";
				} else if (!isUnsigned && isShowSign) {
					signStr = "+";
				} else if (!isUnsigned && isSpaceSign) {
					signStr = " ";
				}

				let prefix = "";
				if (isAlternate) {
					if (radix === 8) {
						if (!s.startsWith("0")) {
							prefix = "0";
						}
					} else if (radix === 16 && numVal !== 0n) {
						prefix = isUppercase ? "0X" : "0x";
					}
				}

				let fullStr: string;
				if (precision < 0 && isZeroPad && !isLeftAlign) {
					let zeros = "";
					while (
						signStr.length + prefix.length + zeros.length + s.length <
						width
					) {
						zeros += "0";
					}
					fullStr = signStr + prefix + zeros + s;
				} else {
					fullStr = signStr + prefix + s;
				}
				if (fullStr.length < width) {
					const padLen = width - fullStr.length;
					const pad = " ".repeat(padLen);
					fullStr = isLeftAlign ? fullStr + pad : pad + fullStr;
				}

				writeStdout(fullStr);
				return fullStr.length;
			},

			print_double: (
				val: number,
				width: number,
				precision: number,
				mode: number,
				flags: number,
			): number => {
				const isAlternate = (flags & 64) !== 0;
				let s = "";
				if (mode === 0) {
					const prec = precision >= 0 ? precision : 6;
					s = formatGExact(val, prec, isAlternate);
				} else if (mode === 1) {
					const p = precision >= 0 ? precision : 6;
					s = roundToEvenExact(val, p);
					if (isAlternate && Number.isFinite(val) && !s.includes(".")) {
						s += ".";
					}
				} else if (mode === 2) {
					const p = precision >= 0 ? precision : 6;
					s = formatExpExact(val, p);
					if (isAlternate && Number.isFinite(val) && !s.includes(".")) {
						const e = s.indexOf("e");
						s = e >= 0 ? `${s.slice(0, e)}.${s.slice(e)}` : `${s}.`;
					}
				} else {
					throw new Error(`unknown float format mode: ${mode}`);
				}
				if ((flags & 16) !== 0) {
					s = s.toUpperCase();
				}
				const isZeroPad = (flags & 1) !== 0;
				const isLeftAlign = (flags & 2) !== 0;
				const isShowSign = (flags & 4) !== 0;
				const isSpaceSign = (flags & 8) !== 0;
				let sign = "";
				if (s.startsWith("-")) {
					sign = "-";
					s = s.slice(1);
				} else if (isShowSign) {
					sign = "+";
				} else if (isSpaceSign) {
					sign = " ";
				}
				if (width > 0 && sign.length + s.length < width) {
					const padLen = width - sign.length - s.length;
					if (isZeroPad && !isLeftAlign && Number.isFinite(val)) {
						s = sign + "0".repeat(padLen) + s;
					} else {
						const pad = " ".repeat(padLen);
						s = isLeftAlign ? sign + s + pad : pad + sign + s;
					}
				} else {
					s = sign + s;
				}
				writeStdout(s);
				return s.length;
			},

			read_char: (): number => {
				if (typeof stdin === "function") {
					return stdin();
				} else if (typeof stdin === "string") {
					if (inPos < stdin.length) {
						return stdin.charCodeAt(inPos++);
					}
					return -1;
				}
				return -1;
			},

			exit: (code: number): void => {
				if (onExit) {
					onExit(code);
				} else if (
					typeof process !== "undefined" &&
					typeof process.exit === "function"
				) {
					process.exit(code);
				}
				// A browser with no onExit hook has nothing to stop the wasm program;
				// browser hosts should supply onExit (the playground throws from it).
			},
		},
	};

	return {
		imports,
		getStdout: () => stdoutBuffer,
	};
}
