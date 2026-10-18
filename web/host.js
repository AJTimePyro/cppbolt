(function (root, factory) {
    if (typeof module === 'object' && module.exports) {
        module.exports = factory();
    } else {
        root.CppWasmHost = factory();
        root.createWasmImports = root.CppWasmHost.createWasmImports;
    }
}(typeof self !== 'undefined' ? self : this, function () {

    function roundToEvenExact(vNum, prec) {
        if (isNaN(vNum)) return "nan";
        if (vNum === Infinity) return "inf";
        if (vNum === -Infinity) return "-inf";
        if (Object.is(vNum, -0)) {
            return "-0" + (prec > 0 ? "." + "0".repeat(prec) : "");
        }
        if (vNum === 0) {
            return "0" + (prec > 0 ? "." + "0".repeat(prec) : "");
        }
        const isNeg = vNum < 0;
        const v = Math.abs(vNum);

        const buf = new ArrayBuffer(8);
        const view = new DataView(buf);
        view.setFloat64(0, v, false);
        const bits = view.getBigUint64(0, false);
        const rawExp = Number((bits >> 52n) & 0x7ffn);
        const rawMantissa = bits & 0xfffffffffffffn;

        let mantissa, exp;
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

        let num, den;
        if (expP >= 0n) {
            num = mantissa * (2n ** expP) * p5;
            den = 1n;
        } else {
            num = mantissa * p5;
            den = 2n ** (-expP);
        }

        const Q = num / den;
        const R = num % den;
        const twoR = R * 2n;

        let rounded;
        if (twoR < den) {
            rounded = Q;
        } else if (twoR > den) {
            rounded = Q + 1n;
        } else {
            rounded = (Q % 2n === 0n) ? Q : Q + 1n;
        }

        let s = rounded.toString();
        if (prec > 0) {
            while (s.length <= prec) {
                s = "0" + s;
            }
            const intPart = s.slice(0, s.length - prec);
            const fracPart = s.slice(s.length - prec);
            s = intPart + "." + fracPart;
        }
        return (isNeg ? "-" : "") + s;
    }

    function formatExpExact(vNum, prec) {
        if (isNaN(vNum)) return "nan";
        if (vNum === Infinity) return "inf";
        if (vNum === -Infinity) return "-inf";
        if (Object.is(vNum, -0)) {
            let zeros = prec > 0 ? "." + "0".repeat(prec) : "";
            return "-0" + zeros + "e+00";
        }
        if (vNum === 0) {
            let zeros = prec > 0 ? "." + "0".repeat(prec) : "";
            return "0" + zeros + "e+00";
        }
        const isNeg = vNum < 0;
        let v = Math.abs(vNum);
        let exp = Math.floor(Math.log10(v));

        const buf = new ArrayBuffer(8);
        const view = new DataView(buf);
        view.setFloat64(0, v, false);
        const bits = view.getBigUint64(0, false);
        const rawExp = Number((bits >> 52n) & 0x7ffn);
        const rawMantissa = bits & 0xfffffffffffffn;

        let mantissa, bexp;
        if (rawExp === 0) {
            mantissa = rawMantissa;
            bexp = -1022 - 52;
        } else {
            mantissa = rawMantissa | (1n << 52n);
            bexp = rawExp - 1023 - 52;
        }

        let Q, R, den;
        while (true) {
            let shift = prec - exp;
            const S = BigInt(shift);
            let num;
            if (S >= 0n) {
                const p5 = 5n ** S;
                const expP = BigInt(bexp) + S;
                if (expP >= 0n) {
                    num = mantissa * (2n ** expP) * p5;
                    den = 1n;
                } else {
                    num = mantissa * p5;
                    den = 2n ** (-expP);
                }
            } else {
                const negS = -S;
                const p5 = 5n ** negS;
                const expP = BigInt(bexp) - negS;
                if (expP >= 0n) {
                    num = mantissa * (2n ** expP);
                    den = p5;
                } else {
                    num = mantissa;
                    den = p5 * (2n ** (-expP));
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

        let twoR = R * 2n;
        let rounded;
        if (twoR < den) {
            rounded = Q;
        } else if (twoR > den) {
            rounded = Q + 1n;
        } else {
            rounded = (Q % 2n === 0n) ? Q : Q + 1n;
        }

        if (rounded >= 10n ** BigInt(prec + 1)) {
            rounded = rounded / 10n;
            exp++;
        }

        let s = rounded.toString();
        while (s.length < prec + 1) {
            s = "0" + s;
        }
        let mStr = s[0] + (prec > 0 ? "." + s.slice(1) : "");
        let expSign = exp >= 0 ? "+" : "-";
        let expVal = Math.abs(exp).toString().padStart(2, "0");
        return (isNeg ? "-" : "") + mStr + "e" + expSign + expVal;
    }

    function formatGExact(vNum, prec) {
        if (isNaN(vNum)) return "nan";
        if (vNum === Infinity) return "inf";
        if (vNum === -Infinity) return "-inf";
        if (Object.is(vNum, -0)) return "-0";
        if (vNum === 0) return "0";
        if (!prec || prec <= 0) prec = 1;
        let v = Math.abs(vNum);
        let exp = Math.floor(Math.log10(v));
        let res;
        if (exp < -4 || exp >= prec) {
            res = formatExpExact(vNum, prec - 1);
            let [m, e] = res.split("e");
            if (m.includes(".")) {
                m = m.replace(/0+$/, "").replace(/\.$/, "");
            }
            res = m + "e" + e;
        } else {
            let pAfterDot = Math.max(0, prec - 1 - exp);
            res = roundToEvenExact(vNum, pAfterDot);
            if (res.includes(".")) {
                res = res.replace(/0+$/, "").replace(/\.$/, "");
            }
        }
        return res;
    }

    function createWasmImports(onStdout, getStdin, onExit) {
        let stdoutBuffer = "";
        let inPos = 0;

        const writeStdout = (str) => {
            stdoutBuffer += str;
            if (onStdout) {
                onStdout(str);
            }
        };

        const imports = {
            env: {
                print_char: (ch) => {
                    writeStdout(String.fromCharCode(ch));
                },
                print_int: (val, radix, width, precision, flags) => {
                    const isZeroPad = (flags & 1) !== 0;
                    const isLeftAlign = (flags & 2) !== 0;
                    const isShowSign = (flags & 4) !== 0;
                    const isSpaceSign = (flags & 8) !== 0;
                    const isUppercase = (flags & 16) !== 0;
                    const isUnsigned = (flags & 32) !== 0;

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
                                s = "0" + s;
                            }
                        }
                    }

                    let signStr = "";
                    if (isNeg) {
                        signStr = "-";
                    } else if (isShowSign) {
                        signStr = "+";
                    } else if (isSpaceSign) {
                        signStr = " ";
                    }

                    if (precision < 0 && isZeroPad && !isLeftAlign) {
                        while (signStr.length + s.length < width) {
                            s = "0" + s;
                        }
                    }

                    let fullStr = signStr + s;
                    if (fullStr.length < width) {
                        const padLen = width - fullStr.length;
                        const pad = " ".repeat(padLen);
                        fullStr = isLeftAlign ? (fullStr + pad) : (pad + fullStr);
                    }

                    writeStdout(fullStr);
                },
                print_double: (val, width, precision, mode, flags) => { 
                    let s = "";
                    if (mode === 0) {
                        const prec = precision >= 0 ? precision : 6;
                        s = formatGExact(val, prec);
                    } else if (mode === 1) {
                        const p = precision >= 0 ? precision : 6;
                        s = roundToEvenExact(val, p);
                    } else if (mode === 2) {
                        const p = precision >= 0 ? precision : 6;
                        s = formatExpExact(val, p);
                    } else {
                        s = String(val);
                    }
                    const isLeftAlign = (flags & 2) !== 0;
                    if (width > 0 && s.length < width) {
                        const pad = " ".repeat(width - s.length);
                        s = isLeftAlign ? (s + pad) : (pad + s);
                    }
                    writeStdout(s);
                },
                read_char: () => {
                    if (typeof getStdin === 'function') {
                        return getStdin();
                    } else if (typeof getStdin === 'string') {
                        if (inPos < getStdin.length) {
                            return getStdin.charCodeAt(inPos++);
                        }
                        return -1;
                    }
                    return -1;
                },
                exit: (code) => {
                    if (onExit) {
                        onExit(code);
                    } else if (typeof process !== 'undefined' && process.exit) {
                        process.exit(code);
                    }
                }
            }
        };

        return {
            imports,
            getStdout: () => stdoutBuffer,
            flushStdout: () => {
                const s = stdoutBuffer;
                stdoutBuffer = "";
                return s;
            }
        };
    }

    return {
        createWasmImports,
        roundToEvenExact,
        formatExpExact,
        formatGExact
    };
}));

if (typeof require !== "undefined" && typeof module !== "undefined" && require.main === module) {
    const fs = require("fs");
    const wasmPath = process.argv[2];
    let inputStr = process.argv[3];
    if (inputStr === undefined) {
        try {
            if (!process.stdin.isTTY) {
                inputStr = fs.readFileSync(0, "utf-8");
            }
        } catch (_) {}
    }
    inputStr = inputStr || "";
    if (!wasmPath) {
        console.error("Usage: node host.js <file.wasm> [stdin]");
        process.exit(1);
    }
    const wasmBytes = fs.readFileSync(wasmPath);
    const { createWasmImports } = module.exports;
    const wasmHost = createWasmImports(
        null,
        inputStr,
        (code) => {
            process.stdout.write(wasmHost.getStdout());
            process.exit(code);
        }
    );
    WebAssembly.instantiate(wasmBytes, wasmHost.imports)
        .then(({ instance }) => {
            let ret = 0;
            try {
                if (instance.exports.main) {
                    ret = instance.exports.main();
                } else if (instance.exports._start) {
                    ret = instance.exports._start();
                }
            } catch (trapErr) {
                console.error("WASM_TRAP:", trapErr.message);
                process.exit(134);
            }
            process.stdout.write(wasmHost.getStdout());
            if (ret !== undefined && ret !== 0) {
                process.exit(ret);
            }
        })
        .catch(err => {
            console.error("WASM Execution Error:", err);
            process.exit(1);
        });
}
