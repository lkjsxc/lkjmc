package com.lkjmc.common.result;

public sealed interface DecisionResult<T> permits DecisionResult.Ok, DecisionResult.Err {
    record Ok<T>(T value) implements DecisionResult<T> {}
    record Err<T>(String code, String message) implements DecisionResult<T> {}
}
