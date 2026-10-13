// harness.cpp - text-protocol driver for the ORIGINAL OpenKey C++ engine.
//
// Reads one command per line on stdin, writes one line per K/M/E command on
// stdout (format documented in crates/engine/examples/gen_corpus.rs).
// Engine sources are compiled unmodified from third_party/OpenKey; this file
// only supplies the config globals the host application normally defines.

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fcntl.h>
#include <io.h>
#include <string>
#include <vector>

#include "Engine.h"
#include "Macro.h"

// ---- config globals normally defined by the host app (AppDelegate/OpenKey.cpp) ----
int vLanguage = 1;
int vInputType = 0;
int vFreeMark = 0;
int vCodeTable = 0;
int vSwitchKeyStatus = 0;
int vCheckSpelling = 1;
int vUseModernOrthography = 0;
int vQuickTelex = 0;
int vRestoreIfWrongSpelling = 1;
int vFixRecommendBrowser = 0;
int vUseMacro = 0;
int vUseMacroInEnglishMode = 0;
int vAutoCapsMacro = 0;
int vUseSmartSwitchKey = 0;
int vUpperCaseFirstChar = 0;
int vTempOffSpelling = 0;
int vAllowConsonantZFWJ = 0;
int vQuickStartConsonant = 0;
int vQuickEndConsonant = 0;
int vRememberCode = 0;
int vOtherLanguage = 0;
int vTempOffOpenKey = 0;

// Defined (non-static) in Engine.cpp but not declared in any header.
extern vKeyHookState HookState;

namespace {

struct ConfigVar {
    const char* name;
    int* ptr;
};

const ConfigVar kConfigVars[] = {
    {"vLanguage", &vLanguage},
    {"vInputType", &vInputType},
    {"vFreeMark", &vFreeMark},
    {"vCodeTable", &vCodeTable},
    {"vSwitchKeyStatus", &vSwitchKeyStatus},
    {"vCheckSpelling", &vCheckSpelling},
    {"vUseModernOrthography", &vUseModernOrthography},
    {"vQuickTelex", &vQuickTelex},
    {"vRestoreIfWrongSpelling", &vRestoreIfWrongSpelling},
    {"vFixRecommendBrowser", &vFixRecommendBrowser},
    {"vUseMacro", &vUseMacro},
    {"vUseMacroInEnglishMode", &vUseMacroInEnglishMode},
    {"vAutoCapsMacro", &vAutoCapsMacro},
    {"vUseSmartSwitchKey", &vUseSmartSwitchKey},
    {"vUpperCaseFirstChar", &vUpperCaseFirstChar},
    {"vTempOffSpelling", &vTempOffSpelling},
    {"vAllowConsonantZFWJ", &vAllowConsonantZFWJ},
    {"vQuickStartConsonant", &vQuickStartConsonant},
    {"vQuickEndConsonant", &vQuickEndConsonant},
    {"vRememberCode", &vRememberCode},
    {"vOtherLanguage", &vOtherLanguage},
    {"vTempOffOpenKey", &vTempOffOpenKey},
};

std::string g_out;                          // buffered stdout
std::vector<std::string> g_macro_texts;     // texts added since last MACRO_CLEAR
unsigned long g_overflow = 0;               // ncc > MAX_BUFF seen (should stay 0)

void flush_out() {
    fwrite(g_out.data(), 1, g_out.size(), stdout);
    g_out.clear();
}

void die(const char* msg, const char* line_start, const char* line_end) {
    fprintf(stderr, "harness: %s: '%.*s'\n", msg, (int)(line_end - line_start), line_start);
    exit(2);
}

// Emit the HookState line: "<code> <bpc> <ncc> <ext> | <d..> | <m..>".
// Macro data is printed only when code == vReplaceMaro (4).
void emit_hook_state() {
    const vKeyHookState& s = HookState;
    char buf[64];
    snprintf(buf, sizeof buf, "%u %u %u %u |", (unsigned)s.code, (unsigned)s.backspaceCount,
             (unsigned)s.newCharCount, (unsigned)s.extCode);
    g_out += buf;
    unsigned n = s.newCharCount;
    if (n > MAX_BUFF) {
        n = MAX_BUFF;
        ++g_overflow;
    }
    for (unsigned i = 0; i < n; i++) {
        snprintf(buf, sizeof buf, " %x", (unsigned)s.charData[i]);
        g_out += buf;
    }
    g_out += " |";
    if ((unsigned)s.code == 4) {
        for (size_t i = 0; i < s.macroData.size(); i++) {
            snprintf(buf, sizeof buf, " %x", (unsigned)s.macroData[i]);
            g_out += buf;
        }
    }
    g_out += '\n';
    if (g_out.size() > (1u << 20)) flush_out();
}

const char* skip_ws(const char* p, const char* e) {
    while (p < e && (*p == ' ' || *p == '\t')) ++p;
    return p;
}

const char* word_end(const char* p, const char* e) {
    while (p < e && *p != ' ' && *p != '\t') ++p;
    return p;
}

// Parses a decimal integer (optional '-') after skipping blanks. Never reads past e.
long parse_int(const char*& p, const char* e, const char* line_start) {
    p = skip_ws(p, e);
    bool neg = false;
    if (p < e && *p == '-') {
        neg = true;
        ++p;
    }
    if (p >= e || *p < '0' || *p > '9') die("bad integer", line_start, e);
    long v = 0;
    while (p < e && *p >= '0' && *p <= '9') {
        v = v * 10 + (*p - '0');
        ++p;
    }
    return neg ? -v : v;
}

void run_line(const char* p, const char* e) {
    p = skip_ws(p, e);
    if (p >= e || *p == '#') return;
    const char* line_start = p;
    const char* we = word_end(p, e);
    std::string cmd(p, we - p);
    const char* q = we;

    if (cmd == "K" || cmd == "E") {
        long kc = parse_int(q, e, line_start);
        long caps = parse_int(q, e, line_start);
        long ctrl = parse_int(q, e, line_start);
        if (cmd == "K") {
            vKeyHandleEvent(Keyboard, KeyDown, (Uint16)kc, (Uint8)caps, ctrl != 0);
        } else {
            vEnglishMode(KeyDown, (Uint16)kc, caps != 0, ctrl != 0);
        }
        emit_hook_state();
    } else if (cmd == "M") {
        vKeyHandleEvent(Mouse, MouseDown, 0, 0, false);
        emit_hook_state();
    } else if (cmd == "INIT") {
        vKeyInit();
    } else if (cmd == "SET") {
        const char* ns = skip_ws(q, e);
        const char* ne = word_end(ns, e);
        std::string name(ns, ne - ns);
        q = ne;
        long v = parse_int(q, e, line_start);
        bool found = false;
        for (const ConfigVar& var : kConfigVars) {
            if (name == var.name) {
                *var.ptr = (int)v;
                found = true;
                break;
            }
        }
        if (!found) die("unknown config variable", line_start, e);
    } else if (cmd == "SET_SPELL") {
        vSetCheckSpelling();
    } else if (cmd == "TOGGLE_SPELL") {
        vTempOffSpellChecking();
    } else if (cmd == "TEMP_OFF") {
        long v = parse_int(q, e, line_start);
        vTempOffEngine(v != 0);
    } else if (cmd == "NEWSESSION") {
        startNewSession();
    } else if (cmd == "MACRO") {
        const char* ts = skip_ws(q, e);
        const char* te = word_end(ts, e);
        std::string text(ts, te - ts);
        const char* cs = skip_ws(te, e);
        std::string content(cs, e - cs);
        addMacro(text, content);
        g_macro_texts.push_back(text);
    } else if (cmd == "MACRO_CLEAR") {
        for (const std::string& t : g_macro_texts) deleteMacro(t);
        g_macro_texts.clear();
    } else {
        die("unknown command", line_start, e);
    }
}

}  // namespace

int main() {
    _setmode(_fileno(stdin), _O_BINARY);
    _setmode(_fileno(stdout), _O_BINARY);

    std::string input;
    char buf[1 << 16];
    size_t r;
    while ((r = fread(buf, 1, sizeof buf, stdin)) > 0) input.append(buf, r);

    vKeyInit();

    const char* base = input.data();
    const char* end = base + input.size();
    for (const char* p = base; p < end;) {
        const char* nl = static_cast<const char*>(memchr(p, '\n', end - p));
        const char* le = nl ? nl : end;
        const char* ce = le;
        if (ce > p && ce[-1] == '\r') --ce;
        run_line(p, ce);
        p = nl ? nl + 1 : end;
    }

    flush_out();
    fflush(stdout);
    if (g_overflow) fprintf(stderr, "harness: WARNING newCharCount > MAX_BUFF %lu times\n", g_overflow);
    return 0;
}
