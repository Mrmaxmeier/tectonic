/* tectonic/output.c -- functions related to outputting messages
 * Copyright 2016 the Tectonic Project
 * Licensed under the MIT License.
*/

#include "tectonic.h"
#include "internals.h"
#include "xetexd.h"
#include "synctex.h"
#include "core-bridge.h"


void
print_ln(void)
{
    switch (selector) {
    case SELECTOR_TERM_AND_LOG:
        ttstub_output_putc(rust_stdout, '\n');
        ttstub_output_putc(log_file, '\n');
        term_offset = 0;
        file_offset = 0;
        break;
    case SELECTOR_LOG_ONLY:
        ttstub_output_putc(log_file, '\n');
        file_offset = 0;
        break;
    case SELECTOR_TERM_ONLY:
        ttstub_output_putc(rust_stdout, '\n');
        term_offset = 0;
        break;
    case SELECTOR_NO_PRINT:
    case SELECTOR_PSEUDO:
    case SELECTOR_NEW_STRING:
        break;
    default:
        ttstub_output_putc(write_file[selector], '\n');
        break;
    }
}


void
print_raw_char(int32_t s, bool incr_offset)
{
    switch (selector) {
    case SELECTOR_TERM_AND_LOG:
        ttstub_output_putc(rust_stdout, s);
        ttstub_output_putc(log_file, s);
        if (incr_offset) {
            term_offset++;
            file_offset++;
        }
        if (term_offset == max_print_line) {
            ttstub_output_putc(rust_stdout, '\n');
            term_offset = 0;
        }
        if (file_offset == max_print_line) {
            ttstub_output_putc(log_file, '\n');
            file_offset = 0;
        }
        break;
    case SELECTOR_LOG_ONLY:
        ttstub_output_putc(log_file, s);
        if (incr_offset)
            file_offset++;
        if (file_offset == max_print_line)
            print_ln();
        break;
    case SELECTOR_TERM_ONLY:
        ttstub_output_putc(rust_stdout, s);
        if (incr_offset)
            term_offset++;
        if (term_offset == max_print_line)
            print_ln();
        break;
    case SELECTOR_NO_PRINT:
        break;
    case SELECTOR_PSEUDO:
        if (tally < trick_count)
            trick_buf[tally % error_line] = s;
        break;
    case SELECTOR_NEW_STRING:
        if (pool_ptr < pool_size) {
            str_pool[pool_ptr] = s;
            pool_ptr++;
        }
        break;
    default:
        ttstub_output_putc(write_file[selector], s);
        break;
    }
    tally++;
}


void
print_char(int32_t s)
{
    small_number l;

    if ((selector > SELECTOR_PSEUDO) && (!doing_special)) {
        char tmpbuf[6];
        int bufidx = 0;
        write_uchar(tmpbuf, s, &bufidx);
        for (int i = 0; i < bufidx; i++)
            print_raw_char(tmpbuf[i], true);
        return;
    }

    if ( /*252: */ s == INTPAR(new_line_char) /*:252 */ ) {
        if (selector < SELECTOR_PSEUDO) {
            print_ln();
            return;
        }
    }

    if (s < 32 && !doing_special) {
        print_raw_char('^' , true);
        print_raw_char('^' , true);
        print_raw_char(s + 64, true);
    } else if (s < 127) {
        print_raw_char(s, true);
    } else if (s == 127) {
        if (!doing_special) {
            print_raw_char('^' , true);
            print_raw_char('^' , true);
            print_raw_char('?' , true);
        } else {
            print_raw_char(s, true);
        }
    } else if (s < 160 && !doing_special) {
        print_raw_char('^' , true);
        print_raw_char('^' , true);

        l = (s % 256) / 16;
        if (l < 10)
            print_raw_char('0' + l, true);
        else
            print_raw_char('a' + l - 10, true);

        l = s % 16;
        if (l < 10)
            print_raw_char('0' + l, true);
        else
            print_raw_char('a' + l - 10, true);
    } else {
        if (s < 2048) {
            print_raw_char(192 + s / 64, false);
            print_raw_char(128 + s % 64, true);
        } else if (s < 0x10000) {
            print_raw_char(224 + (s / 4096), false);
            print_raw_char(128 + (s % 4096) / 64, false);
            print_raw_char(128 + (s % 64), true);
        } else {
            print_raw_char(240 + (s / 0x40000), false);
            print_raw_char(128 + (s % 0x40000) / 4096, false);
            print_raw_char(128 + (s % 4096) / 64, false);
            print_raw_char(128 + (s % 64), true);
        }
    }
}


void
print(int32_t s)
{
    int32_t nl;

    if (s >= str_ptr)
        return print_cstr("???");
    else if (s < BIGGEST_CHAR) {
        if (s < 0)
            return print_cstr("???");
        else {
            if (selector > SELECTOR_PSEUDO) {
                print_char(s);
                return;
            }

            if ( /*252: */ s == INTPAR(new_line_char) /*:252 */ ) {
                if (selector < SELECTOR_PSEUDO) {
                    print_ln();
                    return;
                }
            }

            nl = INTPAR(new_line_char);
            INTPAR(new_line_char) = -1;
            print_char(s);
            INTPAR(new_line_char) = nl;
            return;
        }
    }

    int32_t pool_idx = s - 0x10000;
    pool_pointer i = str_start[pool_idx];
    while (i < str_start[pool_idx + 1]) {
        uchar_t c = get_uchar(str_pool, &i);
        print_char(c);
    }
}


void
print_cstr(const char* str)
{
    for (unsigned int i = 0; i < strlen(str); i++) {
        print_char(str[i]);
    }
}


void
print_nl(str_number s)
{
    if (((term_offset > 0) && (odd(selector))) || ((file_offset > 0) && (selector >= SELECTOR_LOG_ONLY)))
        print_ln();
    print(s);
}

void
print_nl_cstr(const char* str)
{
    if (((term_offset > 0) && (odd(selector))) || ((file_offset > 0) && (selector >= SELECTOR_LOG_ONLY)))
        print_ln();
    print_cstr(str);
}


void
print_esc(str_number s)
{

    int32_t c = INTPAR(escape_char) /*:251 */ ;

    if (c >= 0 && c <= BIGGEST_USV)
        print_char(c);
    print(s);
}

void
print_esc_cstr(const char* s)
{

    int32_t c = INTPAR(escape_char) /*:251 */ ;

    if (c >= 0 && c <= BIGGEST_USV)
        print_char(c);
    print_cstr(s);
}


static void
print_the_digs(eight_bits k)
{
    while (k > 0) {
        k--;
        if (dig[k] < 10)
            print_char('0'  + dig[k]);
        else
            print_char(55 /*"A" -10 */  + dig[k]);
    }
}


void
print_int(int32_t n)
{
    unsigned char k = 0;
    int32_t m;

    if (n < 0) {
        print_char('-');
        if (n > -100000000L)
            n = -(int32_t) n;
        else {
            m = -1 - n;
            n = m / 10;
            m = (m % 10) + 1;
            k = 1;
            if (m < 10)
                dig[0] = m;
            else {
                dig[0] = 0;
                n++;
            }
        }
    }

    do {
        dig[k] = n % 10;
        n = n / 10;
        k++;
    } while (!(n == 0));

    print_the_digs(k);
}


void
print_cs(int32_t p)
{
    if (p < HASH_BASE) {
        if (p >= SINGLE_BASE) {
            if (p == NULL_CS) {
                print_esc_cstr("csname");
                print_esc_cstr("endcsname");
                print_char(' ');
            } else {
                print_esc(p - SINGLE_BASE);
                if (CAT_CODE(p - SINGLE_BASE) == LETTER)
                    print_char(' ');
            }
        } else if (p < ACTIVE_BASE)
            print_esc_cstr("IMPOSSIBLE.");
        else
            print_char(p - 1);
    } else if (((p >= UNDEFINED_CONTROL_SEQUENCE) && (p <= EQTB_SIZE)) || (p > eqtb_top)) {
        print_esc_cstr("IMPOSSIBLE.");
    } else if (hash[p].s1 >= str_ptr) {
        print_esc_cstr("NONEXISTENT.");
    } else {
        print_esc(hash[p].s1);
        print_char(' ');
    }
}


void
sprint_cs(int32_t p)
{
    if (p < HASH_BASE) {
        if (p < SINGLE_BASE)
            print_char(p - 1);
        else if (p < NULL_CS)
            print_esc(p - SINGLE_BASE);
        else {
            print_esc_cstr("csname");
            print_esc_cstr("endcsname");
        }
    } else
        print_esc(hash[p].s1);
}


void
print_file_name(int32_t n, int32_t a, int32_t e)
{
    bool must_quote = false;
    char quote_char = '"';

    char* sa = gettexstring(a);
    char* sn = gettexstring(n);
    char* se = gettexstring(e);
    int len = strlen(sa) + strlen(sn) + strlen(se);
    char* str = malloc(len + 1);
    sprintf(str, "%s%s%s", sa, sn, se);
    free(sa);
    free(sn);
    free(se);

    for (int i = 0; i < len; i++) {
        if (str[i] == ' ')
            must_quote = true;
        else if (str[i] == '"' || str[i] == '\'')
            must_quote = true;
            quote_char = str[i] == '"' ? '\'' : '"';
    }

    if (must_quote)
        print_char(quote_char);
    for (int i = 0; i < len; i++) {
        if (str[i] == quote_char) {
            print_char(quote_char);
            quote_char = str[i] == '"' ? '\'' : '"';
            print_char(quote_char);
        }
        print_raw_char(str[i], true); // MEH
    }
    if (must_quote)
        print_char(quote_char);

    free(str);
}


void
print_size(int32_t s)
{
    if (s == TEXT_SIZE)
        print_esc_cstr("textfont");
    else if (s == SCRIPT_SIZE)
        print_esc_cstr("scriptfont");
    else
        print_esc_cstr("scriptscriptfont");
}


void
print_write_whatsit(const char* s, int32_t p)
{

    print_esc_cstr(s);

    if (mem[p + 1].b32.s0 < 16)
        print_int(mem[p + 1].b32.s0);
    else if (mem[p + 1].b32.s0 == 16)
        print_char('*');
    else
        print_char('-');
}


void
print_native_word(int32_t p)
{
    int32_t i, c, cc;
    int32_t for_end = mem[p + 4].b16.s1 - 1;

    for (i = 0; i <= for_end; i++) {
        c = NATIVE_NODE_text(p)[i];
        if ((c >= 0xD800) && (c < 0xDC00)) {
            if (i < mem[p + 4].b16.s1 - 1) {
                cc = NATIVE_NODE_text(p)[i + 1];
                if ((cc >= 0xDC00) && (cc < 0xE000)) {
                    c = 0x10000 + (c - 0xD800) * 1024 + (cc - 0xDC00);
                    print_char(c);
                    i++;
                } else
                    print('.');
            } else
                print('.');
        } else
            print_char(c);
    }
}


void
print_sa_num(int32_t q)
{
    int32_t n;

    if (mem[q].b16.s1 < DIMEN_VAL_LIMIT)
        n = mem[q + 1].b32.s1;
    else {
        n = mem[q].b16.s1 % 64;
        q = mem[q].b32.s1;
        n = n + 64 * mem[q].b16.s1;
        q = mem[q].b32.s1;
        n = n + 64 * 64 * (mem[q].b16.s1 + 64 * mem[mem[q].b32.s1].b16.s1);
    }

    print_int(n);
}


void
print_file_line(void)
{
    int32_t level = in_open;

    while ((level > 0) && (full_source_filename_stack[level] == 0))
        level--;

    if (level == 0)
        print_nl_cstr("! ");
    else {
        print_nl_cstr("");
        print(full_source_filename_stack[level]);
        print(':');
        if (level == in_open)
            print_int(line);
        else
            print_int(line_stack[level + 1]);
        print_cstr(": ");
    }
}
/*:1660*/


void
print_two(int32_t n)
{
    n = abs(n) % 100;
    print_char('0' + (n / 10));
    print_char('0' + (n % 10));
}


void
print_hex(int32_t n)
{
    unsigned char k = 0;

    print_char('"');

    do {
        dig[k] = n % 16;
        n = n / 16;
        k++;
    } while (n != 0);

    print_the_digs(k);
}


void
print_roman_int(int32_t n)
{
    int32_t u, v;

    const char* roman_data = "m2d5c2l5x2v5i";
    unsigned char j = 0;
    unsigned char k = 0;
    v = 1000;

    while (true) {
        while (n >= v) {
            print_char(roman_data[j]);
            n = n - v;
        }

        if (n <= 0)
            return;

        k = j + 2;
        u = v / (roman_data[k - 1] - '0');
        if (roman_data[k - 1] == '2' ) {
            k = k + 2;
            u = u / (roman_data[k - 1] - '0');
        }

        if (n + u >= v) {
            print_char(roman_data[k]);
            n = n + u;
        } else {
            j = j + 2;
            v = v / (roman_data[j - 1] - '0');
        }
    }
}


void
print_current_string(void)
{
    pool_pointer j = str_start[str_ptr - 0x10000];

    while (j < pool_ptr) {
        print_char(str_pool[j]);
        j++;
    }
}


void
print_scaled(scaled_t s)
{
    scaled_t delta;

    if (s < 0) {
        print_char('-');
        s = -(int32_t) s;
    }

    print_int(s / 0x10000);
    print_char('.');
    s = 10 * (s % 0x10000) + 5;
    delta = 10;

    do {
        if (delta > 0x10000)
            s = s + 0x8000 - 50000;
        print_char('0'  + (s / 0x10000));
        s = 10 * (s % 0x10000);
        delta = delta * 10;
    } while (s > delta);
}
