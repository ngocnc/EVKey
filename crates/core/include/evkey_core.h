/* EVKey core C ABI. Hand-written to match crates/core/src/ffi.rs (a unit test checks every export is declared).
 * Strings returned as char* are owned by the caller: release with evkey_string_free.
 * EvReaction.text_ptr is owned by the session and valid until the next call on that session. */
#ifndef EVKEY_CORE_H
#define EVKEY_CORE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct EvSession EvSession;

#define EV_RESEND_NONE 0
#define EV_RESEND_CHAR 1 /* resend_value is a UTF-16 unit */
#define EV_RESEND_VK 2   /* resend_value is a Windows virtual-key code */

/* mods bits for evkey_session_on_key */
#define EV_MOD_CTRL 1
#define EV_MOD_ALT 2
#define EV_MOD_SHIFT 4
#define EV_MOD_WIN 8

typedef struct EvReaction {
    bool swallow;
    bool hotkey_toggled;
    bool dummy_key;
    bool translate_requested;
    bool has_edit;
    bool prefix_empty_char;
    uint32_t backspaces;
    uint8_t resend_kind;
    uint16_t resend_value;
    const uint16_t *text_ptr;
    size_t text_len;
} EvReaction;

EvSession *evkey_session_new(const char *dir_utf8);
void evkey_session_free(EvSession *session);
bool evkey_session_on_key(EvSession *session, uint16_t vk, bool down, uint8_t mods, bool caps_lock, EvReaction *out);
void evkey_session_mouse_click(EvSession *session);
void evkey_session_set_foreground(EvSession *session, const char *app_id_utf8);
void evkey_session_set_secure(EvSession *session, bool on);
bool evkey_session_is_english(EvSession *session);
void evkey_session_set_english(EvSession *session, bool on);
void evkey_session_toggle(EvSession *session);

char *evkey_default_dir(void);
char *evkey_settings_load_json(const char *dir_utf8);
char *evkey_settings_save_json(const char *dir_utf8, const char *json_utf8);
void evkey_string_free(char *s);
const char *evkey_version(void);

#ifdef __cplusplus
}
#endif

#endif /* EVKEY_CORE_H */
