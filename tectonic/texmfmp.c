/* texmfmp.c: Hand-coded routines for TeX or Metafont in C.  Originally
   written by Tim Morgan, drawing from other Unix ports of TeX.  This is
   a collection of miscellany, everything that's easier (or only
   possible) to do in C.

   This file is public domain.  */

#include "tectonic.h"
#include "internals.h"
#include "core-bridge.h"
#include "xetexd.h"
#include "XeTeX_ext.h"

#include <time.h> /* For `struct tm'.  Moved here for Visual Studio 2005.  */


static char *last_source_name = NULL;
static int last_lineno;

void
get_date_and_time (int32_t *minutes,  int32_t *day,
                   int32_t *month,  int32_t *year)
{
  struct tm *tmptr;

  time_t myclock = time ((time_t *) 0);
  tmptr = localtime (&myclock);
  *minutes = tmptr->tm_hour * 60 + tmptr->tm_min;
  *day = tmptr->tm_mday;
  *month = tmptr->tm_mon + 1;
  *year = tmptr->tm_year + 1900;
}


static void
checkpool_pointer (pool_pointer pool_ptr, size_t len)
{
    if (pool_ptr + len >= pool_size)
        _tt_abort ("string pool overflow [%i bytes]", (int) pool_size);
}


int
maketexstring(const char *s)
{
  size_t len;
  UInt32 rval;
  const unsigned char *cp = (const unsigned char *)s;

  if (s == NULL || *s == 0)
    return EMPTY_STRING;

  len = strlen(s);
  checkpool_pointer (pool_ptr, len); /* in the XeTeX case, this may be more than enough */

  while ((rval = *(cp++)) != 0) {
    str_pool[pool_ptr++] = rval;
  }

  return make_string();
}


str_number
make_full_name_string(void)
{
  return maketexstring(fullnameoffile);
}


char *
gettexstring (str_number s)
{
  pool_pointer len;

  if (s >= 65536L)
    len = str_start[s + 1 - 65536L] - str_start[s - 65536L];
  else
    len = 0;

  char* name = xmalloc(len + 1);
  if (len > 0)
    memcpy(name, str_pool + str_start[s - 65536L], len);
  name[len] = '\0';

  return name;
}


// This code is ripped from bionic libc's mbtoc32r implementation.
uchar_t
get_uchar(UTF8_code * buf, unsigned int * ptr)
{
  int32_t cp = (int32_t) buf[(*ptr)++];

  unsigned int length, mask, lower_bound;

  if ((cp & 0x80) == 0) {
    mask = 0x7f;
    length = 1;
    lower_bound = 0;
  } else if ((cp & 0xe0) == 0xc0) {
    mask = 0x1f;
    length = 2;
    lower_bound = 0x80;
  } else if ((cp & 0xf0) == 0xe0) {
    mask = 0x0f;
    length = 3;
    lower_bound = 0x800;
  } else if ((cp & 0xf8) == 0xf0) {
    mask = 0x07;
    length = 4;
    lower_bound = 0x10000;
  } else {
    bad_utf8_warning();
    return 0xFFFD;
  }

  // printf("get_uchar[%d]: %.*s\n", length, length, buf + (*ptr) - 1);

  cp &= mask;
  for (int i = 1; i < length; i++) {
    unsigned char c = buf[(*ptr)++];
    if ((c & 0xC0) != 0x80) {
      bad_utf8_warning();
      return 0xFFFD;
    }
    cp = (cp << 6) | (c & 0x3F);
  }

  if (cp < lower_bound) {
    bad_utf8_warning();
    return 0xFFFD;
  }

  return cp;
}

// Ditto for c32rtomb
size_t
write_uchar(UTF8_code * buf, uchar_t c, unsigned int * ptr)
{
  if ((c & ~0x7f) == 0) {
    // Fast path for plain ASCII characters.
    buf[(*ptr)++] = c;
    return 1;
  }

  // Determine the number of octets needed to represent this character.
  // We always output the shortest sequence possible. Also specify the
  // first few bits of the first octet, which contains the information
  // about the sequence length.
  uint8_t lead;
  size_t length;
  if ((c & ~0x7f) == 0) {
    lead = 0;
    length = 1;
  } else if ((c & ~0x7ff) == 0) {
    lead = 0xc0;
    length = 2;
  } else if ((c & ~0xffff) == 0) {
    lead = 0xe0;
    length = 3;
  } else if ((c & ~0x1fffff) == 0) {
    lead = 0xf0;
    length = 4;
  } else {
    // TODO
    return 0;
  }
  // Output the octets representing the character in chunks
  // of 6 bits, least significant last. The first octet is
  // a special case because it contains the sequence length
  // information.
  for (size_t i = length - 1; i > 0; i--) {
    buf[i + *ptr] = (c & 0x3f) | 0x80;
    c >>= 6;
  }
  buf[*ptr] = (c & 0xff) | lead;
  *ptr += length;
  return length;
}

static int
compare_paths (const char *p1, const char *p2)
{
  int ret;
  while (
         (((ret = (*p1 - *p2)) == 0) && (*p2 != 0))
                || (IS_DIR_SEP(*p1) && IS_DIR_SEP(*p2))) {
       p1++, p2++;
  }
  ret = (ret < 0 ? -1 : (ret > 0 ? 1 : 0));
  return ret;
}


bool
is_new_source (str_number srcfilename, int lineno)
{
  char *name = gettexstring(srcfilename);
  return (compare_paths(name, last_source_name) != 0 || lineno != last_lineno);
}


void
remember_source_info (str_number srcfilename, int lineno)
{
  free(last_source_name);
  last_source_name = gettexstring(srcfilename);
  last_lineno = lineno;
}


pool_pointer
make_src_special (str_number srcfilename, int lineno)
{
  pool_pointer oldpool_ptr = pool_ptr;
  char *filename = gettexstring(srcfilename);
  /* FIXME: Magic number. */
  char buf[40];
  char *s = buf;

  /* Always put a space after the number, which makes things easier
   * to parse.
   */
  sprintf (buf, "src:%d ", lineno);

  if (pool_ptr + strlen(buf) + strlen(filename) >= (size_t)pool_size)
      _tt_abort ("string pool overflow");

  s = buf;
  while (*s)
    str_pool[pool_ptr++] = *s++;

  s = filename;
  while (*s)
    str_pool[pool_ptr++] = *s++;

  return (oldpool_ptr);
}

/* Converts any given string in into an allowed PDF string which is
 * hexadecimal encoded;
 * sizeof(out) should be at least lin*2+1.
 */
static void
convertStringToHexString(const char *in, char *out, int lin)
{
    static const char hexchars[] = "0123456789ABCDEF";
    int i, j;
    j = 0;

    for (i = 0; i < lin; i++) {
        unsigned char c = (unsigned char) in[i];
        out[j++] = hexchars[(c >> 4) & 0xF];
        out[j++] = hexchars[c & 0xF];
    }
    out[j] = '\0';
}

#define DIGEST_SIZE 16

void getmd5sum(str_number s, bool file)
{
    char digest[DIGEST_SIZE];
    char outbuf[2 * DIGEST_SIZE + 1];
    char *xname;
    int ret, i;

    xname = gettexstring (s);

    if (file)
        ret = ttstub_get_file_md5 (xname, digest);
    else
        ret = ttstub_get_data_md5 (xname, strlen (xname), digest);

    free(xname);
    if (ret)
        return;

    if (pool_ptr + 2 * DIGEST_SIZE >= pool_size) {
        /* error by str_toks that calls str_room(1) */
        return;
    }

    convertStringToHexString((char *) digest, outbuf, DIGEST_SIZE);
    for (i = 0; i < 2 * DIGEST_SIZE; i++)
        str_pool[pool_ptr++] = (uint16_t)outbuf[i];
}
