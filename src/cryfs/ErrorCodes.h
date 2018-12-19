#pragma once
#ifndef MESSMER_CRYFSCLI_EXITCODES_H
#define MESSMER_CRYFSCLI_EXITCODES_H

namespace cryfs {

enum class ErrorCode : int {
  Success = 0,

  // An error happened that doesn't have an error code associated with it
  UnspecifiedError = 1,

  // The command line arguments are invalid.
  InvalidArguments = 10,

  // Couldn't load config file. Probably the password is wrong
  WrongPassword = 11,

  // Password cannot be empty
  EmptyPassword = 12,

  // The file system format is too new for this CryFS version. Please update your CryFS version.
  TooNewFilesystemFormat = 13,

  // The file system format is too old for this CryFS version. Run with --allow-filesystem-upgrade to upgrade it.
  TooOldFilesystemFormat = 14,

  // The file system uses a different cipher than the one specified on the command line using the --cipher argument.
  WrongCipher = 15,

  // Base directory doesn't exist or is inaccessible (i.e. not read or writable or not a directory)
  InaccessibleBaseDir = 16,

  // Mount directory doesn't exist or is inaccessible (i.e. not read or writable or not a directory)
  InaccessibleMountDir = 17,

  // Base directory can't be a subdirectory of the mount directory
  BaseDirInsideMountDir = 18,

  // Something's wrong with the file system.
  InvalidFilesystem = 19,

  // The filesystem id in the config file is different to the last time we loaded a filesystem from this basedir. This could mean an attacker replaced the file system with a different one. You can pass the --allow-replaced-filesystem option to allow this.
  FilesystemIdChanged = 20,

  // The filesystem encryption key differs from the last time we loaded this filesystem. This could mean an attacker replaced the file system with a different one. You can pass the --allow-replaced-filesystem option to allow this.
  EncryptionKeyChanged = 21,

  // The command line options and the file system disagree on whether missing blocks should be treated as integrity violations.
  FilesystemHasDifferentIntegritySetup = 22,

  // File system is in single-client mode and can only be used from the client that created it.
  SingleClientFileSystem = 23,

  // A previous run of the file system detected an integrity violation. Preventing access to make sure the user notices. The file system will be accessible again after the user deletes the integrity state file.
  IntegrityViolationOnPreviousRun = 24,

  // An integrity violation was detected and the file system unmounted to make sure the user notices.
  IntegrityViolation = 25
};

inline int exitCode(ErrorCode code) {
  return static_cast<int>(code);
}

}

#endif
