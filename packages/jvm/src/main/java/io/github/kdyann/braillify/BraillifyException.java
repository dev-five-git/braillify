package io.github.kdyann.braillify;

/** Thrown when input cannot be converted according to the braillify rules. */
public class BraillifyException extends RuntimeException {
    /**
     * Creates an input/conversion error with a message.
     * @param message error detail
     */
    public BraillifyException(String message) {
        super(message);
    }

    /**
     * Creates an input/conversion error with a message and cause.
     * @param message error detail
     * @param cause underlying failure
     */
    public BraillifyException(String message, Throwable cause) {
        super(message, cause);
    }
}
