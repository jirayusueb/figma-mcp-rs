import type { ComponentProps } from "solid-js"
import { splitProps } from "solid-js"

import { cx } from "@/lib/cva"

export type CardProps = ComponentProps<"div">

export const Card = (props: CardProps) => {
  const [, rest] = splitProps(props, ["class"])

  return (
    <div
      data-slot="card"
      class={cx(
        "bg-card text-card-foreground flex flex-col gap-6 border py-6",
        props.class,
      )}
      {...rest}
    />
  )
}

export type CardContentProps = ComponentProps<"div">

export const CardContent = (props: CardContentProps) => {
  const [, rest] = splitProps(props, ["class"])

  return <div data-slot="card-content" class={cx("px-6", props.class)} {...rest} />
}
