# Copyright (c) 2023 The Nimbus Authors. All rights reserved.
#
# The use of this source code is governed by a BSD-style
# license that can be found in the LICENSE file.

def _multiplatform_binary_transition_impl(_, attr):
    return {
        triple: {"//command_line_option:platforms": str(platform)}
        for platform, triple in attr.target_platforms.items()
    }

_multiplatform_binary_transition = transition(
    implementation = _multiplatform_binary_transition_impl,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

def _multiplatform_binary_impl(ctx):
    output_files = []

    for name, target in ctx.split_attr.binary.items():
        output_file = ctx.actions.declare_file(name)
        target_file = target[DefaultInfo].files_to_run.executable
        ctx.actions.symlink(output = output_file, target_file = target_file, is_executable = True)
        output_files.append(output_file)

    runfiles = ctx.runfiles().merge_all([b[DefaultInfo].default_runfiles for b in ctx.attr.binary])

    return [DefaultInfo(files = depset(output_files), runfiles = runfiles)]

multiplatform_binary = rule(
    implementation = _multiplatform_binary_impl,
    attrs = {
        "target_platforms": attr.label_keyed_string_dict(
            doc = "A mapping from target platform to output binary name.",
            mandatory = True,
        ),
        "binary": attr.label(
            mandatory = True,
            cfg = _multiplatform_binary_transition,
            doc = "The binary to be built for multiple platforms.",
        ),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    },
    doc = "Builds a binary for multiple target platforms.",
)
