#ifndef MC_H
#define MC_H

/* Monadnock Cyber GPG — C ABI over mc-core.
 * Results are heap-allocated C strings; free them with mc_string_free.
 * A NULL return means failure. */

#ifdef __cplusplus
extern "C" {
#endif

char *mc_decrypt(const char *keyring, const char *ciphertext);
char *mc_encrypt(const char *keyring, const char *plaintext, const char *recipient);
char *mc_sign(const char *keyring, const char *text, const char *signer);
char *mc_list_json(const char *keyring);
char *mc_generate(const char *keyring, const char *userid);
char *mc_import(const char *keyring, const char *armored);
char *mc_encrypt_to(const char *keyring, const char *plaintext, const char *emails);
char *mc_missing_keys(const char *keyring, const char *emails);
void mc_string_free(char *s);

#ifdef __cplusplus
}
#endif

#endif /* MC_H */
