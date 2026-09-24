use glow::HasContext;

/// Installs a synchronous `GL_DEBUG_OUTPUT` callback that prints to stderr,
/// tagged with `label` so multi-context logs stay distinguishable. Call only
/// while `gl`'s underlying context is current on this thread. The callback
/// is registered against whichever context is bound at this call.
pub fn install(gl: &mut glow::Context, label: &'static str) {
    unsafe {
        gl.enable(glow::DEBUG_OUTPUT);
        gl.enable(glow::DEBUG_OUTPUT_SYNCHRONOUS);
        gl.debug_message_callback(move |source, msg_type, id, severity, message| {
            // NOTIFICATION severity is the driver narrating routine
            // decisions (e.g. NVIDIA's "will use VIDEO memory as the
            // source for buffer object operations", once per buffer, every
            // frame), not a problem. Printing those drowns out the actual
            // GL_INVALID_OPERATION/GL_INVALID_VALUE errors this callback
            // exists to surface.
            if severity == glow::DEBUG_SEVERITY_NOTIFICATION {
                return;
            }
            opendrop_core::log_error!(
                "[gl:{label}] source=0x{source:x} type=0x{msg_type:x} id={id} severity=0x{severity:x}: {message}"
            );
        });
    }
}
