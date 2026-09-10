import type { ComponentProps, ValidComponent } from "solid-js"
import { splitProps } from "solid-js"
import { Root as ButtonPrimitive } from "@kobalte/core/button"
import type { VariantProps } from "cva"

import { cva } from "@/lib/cva"

export const buttonVariants = cva({
  base: [
    "inline-flex items-center justify-center gap-2 whitespace-nowrap border border-primary font-mono text-sm font-semibold uppercase tracking-[0.1em] transition-colors [&_svg:not([class*=size-])]:size-4 shrink-0 outline-none",
    "disabled:pointer-events-none disabled:opacity-55",
    "[&_svg]:pointer-events-none [&_svg]:shrink-0",
    "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring",
    "aria-[invalid]:border-destructive",
  ],

  variants: {
    variant: {
      default:
        "bg-primary text-primary-foreground hover:bg-transparent hover:text-primary",
      destructive:
        "border-destructive bg-destructive text-white hover:bg-transparent hover:text-destructive",
      outline:
        "border-input bg-background hover:border-ring hover:text-primary",
      secondary:
        "border-transparent bg-secondary text-secondary-foreground hover:bg-transparent hover:border-rule-strong",
      ghost:
        "border-transparent hover:bg-accent hover:text-accent-foreground",
      link: "border-transparent text-primary underline-offset-[3px] hover:underline",
    },
    size: {
      default: "h-9 px-4 py-2 has-[>svg]:px-3",
      sm: "h-8 gap-1.5 px-3 has-[>svg]:px-2.5",
      lg: "h-10 px-6 has-[>svg]:px-4",
      icon: "size-9",
      "icon-sm": "size-8",
      "icon-lg": "size-10",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "default",
  },
})

export type ButtonProps<T extends ValidComponent = "button"> = ComponentProps<
  typeof ButtonPrimitive<T>
> &
  VariantProps<typeof buttonVariants>

export const Button = <T extends ValidComponent = "button">(
  props: ButtonProps<T>,
) => {
  const [, rest] = splitProps(props as ButtonProps, [
    "class",
    "variant",
    "size",
  ])

  return (
    <ButtonPrimitive
      data-slot="button"
      class={buttonVariants({
        variant: props.variant,
        size: props.size,
        class: props.class,
      })}
      {...rest}
    />
  )
}
