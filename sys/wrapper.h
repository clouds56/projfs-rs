#include <windows.h>
#include <WinError.h>
#include "projectedfslib.h"

enum IO_ERROR {
  OK = S_OK,
  FILE_NOT_FOUND = ERROR_FILE_NOT_FOUND,
  IO_PENDING = ERROR_IO_PENDING,
  INSUFFICIENT_BUFFER = ERROR_INSUFFICIENT_BUFFER,
};
