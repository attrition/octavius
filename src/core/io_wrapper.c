#include <stdio.h>
#include <stdarg.h>

int io_fprintf(void *stream, const char *format, ...)
{
    va_list args;
    va_start(args, format);
    int result = vfprintf((FILE *)stream, format, args);
    va_end(args);
    return result;
}
