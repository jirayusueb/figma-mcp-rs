import type { ComponentProps, ValidComponent } from "solid-js"
import { splitProps } from "solid-js"
import { TextField as TextFieldPrimitive } from "@kobalte/core/text-field"

import { cx } from "@/lib/cva"

export type TextFieldProps<T extends ValidComponent = "div"> = ComponentProps<
  typeof TextFieldPrimitive<T>
>

export const TextField = <T extends ValidComponent = "div">(
  props: TextFieldProps<T>,
) => {
  const [, rest] = splitProps(props as TextFieldProps, ["class"])

  return (
    <TextFieldPrimitive
      data-slot="text-field"
      class={cx("grid w-full gap-2", props.class)}
      {...rest}
    />
  )
}

export type TextFieldInputProps<T extends ValidComponent = "input"> =
  ComponentProps<typeof TextFieldPrimitive.Input<T>>

export const TextFieldInput = <T extends ValidComponent = "input">(
  props: TextFieldInputProps<T>,
) => {
  const [, rest] = splitProps(props as TextFieldInputProps, ["class"])

  return (
    <TextFieldPrimitive.Input
      data-slot="text-field-input"
      class={cx(
        "placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground border-input flex h-9 border bg-background px-3 py-1 font-mono text-sm transition-[color,box-shadow] disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
        "focus-visible:border-ring focus-visible:outline-2 focus-visible:outline-offset-[-1px] focus-visible:outline-ring",
        "aria-invalid:border-destructive",
        "file:text-foreground file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium",
        props.class,
      )}
      {...rest}
    />
  )
}
