# TODO: generate extension guesser
# On success 
function (slang_guess_ext guessed_ext)
  if (${ARG0} MATCHES "^spirv.*")
    set(${guessed_ext} spv PARENT_SCOPE)
  endif()
endfunction()

# Mandatory arguments:
# * ENTRYPOINTS (execution models must be explicitly
# specified in the shader source)
# * SOURCE (path to single source code file)
# Optional arguments:
# * REFLECT = OFF (whether emit reflection info)
# * TARGET = spirv_1_5 (may be overridden by the ${DEFAULT_SLANG_SHADER_TARGET} gloabl)
# * OUTPUT_DIR = ${CMAKE_CURRENT_BINARY_DIR}
# * OUTPUT_EXT = <guessed based on TARGET> (output file extension; if guessing is impossible,
# the argument must be specified explicitly)
# * SLANG_ARGS = <empty list> (additional arguments to be passed to the compiler)
# * PROFILE = <none> (profile for code generation (glsl_xxx, sm_x_x); may be required by some targets)
function (add_slang_shader OUTPUT)
  # Check that slangc is present
  set(SLANGC_EXECUTABLE "/usr/bin/slangc")
  get_filename_component(TARGET_NAME ${OUTPUT} NAME_WLE)

  if (NOT DEFINED DEFAULT_SLANG_SHADER_TARGET)
    set(DEFAULT_SHADER_TARGET spirv_1_5)
  endif()

  cmake_parse_arguments(PARSE_ARGV 1
    # prefix of parsed args
    SHADER
    # options (args that are either present or not)
    ""
    # one value keywords
    "SOURCE;REFLECT;PROFILE;TARGET;OUTPUT_DIR;OUTPUT_EXT"
    # multi value keywords
    "ENTRYPOINTS;SLANGC_ARGS"
  )

  if (NOT DEFINED SHADER_ENTRYPOINTS OR NOT DEFINED SHADER_SOURCE)
    message(SEND_ERROR "Entrypoints and the source file for the ${ARGV0} shader target must be specified")
  endif()
  if (NOT DEFINED SHADER_TARGET)
    set(SHADER_TARGET ${DEFAULT_SLANG_SHADER_TARGET})
  endif()

  # TODO: implement or burn
  if (NOT DEFINED OUTPUT_EXT)
    # slang_guess_ext()
    # slang_
  endif()

  # get_filename_component(SHADER_SOURCE ${SHADER_SOURCE} )
  cmake_path(ABSOLUTE_PATH SHADER_SOURCE)
  cmake_path(
    ABSOLUTE_PATH OUTPUT
    BASE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    OUTPUT_VARIABLE OUTPUT_PATH
  )

  message(DEBUG "${SHADER_UNPARSED_ARGUMENTS}")
  message(DEBUG "source: ${SHADER_SOURCE}; reflect: ${SHADER_REFLECT};")
  message(DEBUG "profile: ${SHADER_PROFILE}; target: ${SHADER_TARGET};")
  message(DEBUG "entry points: ${SHADER_ENTRY_POINTS}")

  list(APPEND SLANG_ARGS "${SHADER_SOURCE}")
  list(PREPEND SLANG_ARGS "-o" "${OUTPUT_PATH}")
  list(PREPEND SLANG_ARGS "-target" "${SHADER_TARGET}")
  if (DEFINED SHADER_PROFILE)
    list(PREPEND SLANG_ARGS "-profile" "${SHADER_PROFILE}")
  endif()
  if (${SHADER_REFLECT})
    list(PREPEND SLANG_ARGS "-fspv-reflect")
  endif()
  foreach(EP ${SHADER_ENTRYPOINTS})
    list(PREPEND SLANG_ARGS "-entry" "${EP}")
  endforeach()

  add_custom_command(
          OUTPUT  ${OUTPUT_PATH}
          COMMAND ${SLANGC_EXECUTABLE} ARGS ${SLANG_ARGS}
          WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
          DEPENDS ${SHADER_SOURCE}
          COMMENT "Compiling Slang shader ${OUTPUT}"
          VERBATIM
  )
  add_custom_target(${TARGET_NAME} DEPENDS ${OUTPUT})
endfunction()