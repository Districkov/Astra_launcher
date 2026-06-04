/**
 * ASTRA Launcher — Error Handling System
 * Centralized error types, logging, and user-facing messages
 */

/**
 * Custom error types for the launcher
 */
export enum ErrorType {
  NETWORK = "NETWORK_ERROR",
  VALIDATION = "VALIDATION_ERROR",
  FILE_SYSTEM = "FILE_SYSTEM_ERROR",
  PROCESS = "PROCESS_ERROR",
  UPDATE = "UPDATE_ERROR",
  SERVER = "SERVER_ERROR",
  UNKNOWN = "UNKNOWN_ERROR",
}

/**
 * Custom Error class with type and code
 */
export class LauncherError extends Error {
  constructor(
    public type: ErrorType,
    public code: string,
    message: string,
    public details?: Record<string, any>
  ) {
    super(message);
    this.name = "LauncherError";
  }

  /**
   * Get user-friendly error message based on error type
   */
  getUserMessage(): string {
    const messages: Record<ErrorType, string> = {
      [ErrorType.NETWORK]: "Ошибка сетевого соединения. Проверьте подключение к интернету.",
      [ErrorType.VALIDATION]: "Некорректные данные. Проверьте введённые значения.",
      [ErrorType.FILE_SYSTEM]:
        "Ошибка при работе с файлами. Проверьте доступ к папке приложения.",
      [ErrorType.PROCESS]: "Не удалось запустить процесс. Попробуйте позже.",
      [ErrorType.UPDATE]: "Ошибка при обновлении приложения. Попробуйте позже.",
      [ErrorType.SERVER]: "Ошибка сервера. Попробуйте позже или свяжитесь с поддержкой.",
      [ErrorType.UNKNOWN]: "Неизвестная ошибка. Пожалуйста, перезагрузите приложение.",
    };

    return messages[this.type] || messages[ErrorType.UNKNOWN];
  }

  /**
   * Convert to JSON for logging
   */
  toJSON() {
    return {
      name: this.name,
      type: this.type,
      code: this.code,
      message: this.message,
      userMessage: this.getUserMessage(),
      details: this.details,
      timestamp: new Date().toISOString(),
    };
  }
}

/**
 * Error logger with optional console and external logging
 */
export class ErrorLogger {
  private static isDev = import.meta.env.DEV;

  /**
   * Log error to console (dev) and external service (prod)
   */
  static log(error: LauncherError | Error): void {
    const errorObj =
      error instanceof LauncherError
        ? error
        : new LauncherError(
            ErrorType.UNKNOWN,
            "UNKNOWN",
            error.message,
            { originalError: error }
          );

    if (this.isDev) {
      console.error("🔴 LAUNCHER ERROR:", errorObj.toJSON());
    } else {
      // In production, send to external logging service
      // Example: Sentry, LogRocket, etc.
      this.sendToExternalLogger(errorObj);
    }
  }

  /**
   * Send error to external logging service (stub for future implementation)
   */
  private static sendToExternalLogger(error: LauncherError): void {
    // TODO: Implement external error logging
    // Example:
    // fetch('/api/logs', {
    //   method: 'POST',
    //   body: JSON.stringify(error.toJSON())
    // }).catch(() => {
    //   // Silently fail - don't break the app if logging fails
    // });
  }

  /**
   * Create a network error
   */
  static network(message: string, code: string, details?: Record<string, any>) {
    return new LauncherError(ErrorType.NETWORK, code, message, details);
  }

  /**
   * Create a validation error
   */
  static validation(message: string, code: string, details?: Record<string, any>) {
    return new LauncherError(ErrorType.VALIDATION, code, message, details);
  }

  /**
   * Create a file system error
   */
  static fileSystem(message: string, code: string, details?: Record<string, any>) {
    return new LauncherError(ErrorType.FILE_SYSTEM, code, message, details);
  }

  /**
   * Create a process error
   */
  static process(message: string, code: string, details?: Record<string, any>) {
    return new LauncherError(ErrorType.PROCESS, code, message, details);
  }

  /**
   * Create an update error
   */
  static update(message: string, code: string, details?: Record<string, any>) {
    return new LauncherError(ErrorType.UPDATE, code, message, details);
  }

  /**
   * Create a server error
   */
  static server(message: string, code: string, details?: Record<string, any>) {
    return new LauncherError(ErrorType.SERVER, code, message, details);
  }
}

/**
 * Type-safe error handler hook for try-catch blocks
 */
export function handleError(
  error: unknown,
  context: string = "Unknown"
): LauncherError {
  if (error instanceof LauncherError) {
    ErrorLogger.log(error);
    return error;
  }

  if (error instanceof Error) {
    const err = ErrorLogger.server(error.message, `${context}:${error.name}`);
    ErrorLogger.log(err);
    return err;
  }

  const unknownErr = ErrorLogger.network(
    String(error),
    `${context}:UNKNOWN`
  );
  ErrorLogger.log(unknownErr);
  return unknownErr;
}

/**
 * Async error handler wrapper for promise chains
 */
export async function withErrorHandling<T>(
  promise: Promise<T>,
  context: string
): Promise<T> {
  try {
    return await promise;
  } catch (error) {
    throw handleError(error, context);
  }
}
