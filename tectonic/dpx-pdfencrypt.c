/* This is dvipdfmx, an eXtended version of dvipdfm by Mark A. Wicks.

    Copyright (C) 2002-2016 by Jin-Hwan Cho and Shunsaku Hirata,
    the dvipdfmx project team.

    This program is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 2 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program; if not, write to the Free Software
    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA 02111-1307 USA.
*/

#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "core-bridge.h"
#include "dpx-dvipdfmx.h"
#include "dpx-error.h"
#include "dpx-mem.h"
#include "dpx-numbers.h"
#include "dpx-pdfobj.h"
#include "dpx-system.h"
#include "dpx-unicode.h"

#include "dpx-pdfencrypt.h"

unsigned char ID[16];

// NOTE: updating this string will change the PDF ID in output PDFs
#define PRODUCER \
"xdvipdfmx-0.1, Copyright 2002-2015 by Jin-Hwan Cho, Matthias Franz, and Shunsaku Hirata"

// NOTE: this is only used to set the PDF ID
void
pdf_enc_compute_id_string (const char *dviname, const char *pdfname)
{
  char *date_string;
  struct tm *bd_time;

  assert (dviname && pdfname);

  date_string = NEW(15, char);
  bd_time = gmtime(&source_date_epoch);
  sprintf(date_string, "%04d%02d%02d%02d%02d%02d",
          bd_time->tm_year + 1900, bd_time->tm_mon + 1, bd_time->tm_mday,
          bd_time->tm_hour, bd_time->tm_min, bd_time->tm_sec);

  size_t len = strlen(date_string) + strlen(PRODUCER) + strlen(dviname) + strlen(pdfname);
  char* databuf = NEW(len+1, char);
  sprintf(databuf, "%s%s%s%s", date_string, PRODUCER, dviname, pdfname);
  ttstub_get_data_md5(databuf, len, (char*) ID);
  free(date_string);
  free(databuf);
}

pdf_obj *pdf_enc_id_array (void)
{
  pdf_obj *id = pdf_new_array();

  pdf_add_array(id, pdf_new_string(ID, 16));
  pdf_add_array(id, pdf_new_string(ID, 16));

  return id;
}
