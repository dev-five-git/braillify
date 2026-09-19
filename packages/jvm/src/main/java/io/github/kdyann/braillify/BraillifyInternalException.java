package io.github.kdyann.braillify;

/** Thrown when the native bridge fails independently of the caller's input. */
public final class BraillifyInternalException extends RuntimeException {
    /**
     * Creates an internal native-bridge error with a message.
     * @param message error detail
     */
    public BraillifyInternalException(String message) {
        super(message);
    }

    /**
     * Creates an internal native-bridge error with a message and cause.
     * @param message error detail
     * @param cause underlying failure
     */
    public BraillifyInternalException(String message, Throwable cause) {
        super(message, cause);
    }
}
