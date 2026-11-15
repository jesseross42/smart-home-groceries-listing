/**
 * Shadcn-style tooltip on Radix: a short note shown on hover or keyboard
 * focus of its trigger.
 */
import { Tooltip as TooltipPrimitive } from "radix-ui";
import type * as React from "react";

import { cn } from "#/lib/utils";

/** Groups tooltips so moving between them opens the next one at once. */
export function TooltipProvider({
	delayDuration = 0,
	...props
}: React.ComponentProps<typeof TooltipPrimitive.Provider>) {
	return <TooltipPrimitive.Provider delayDuration={delayDuration} {...props} />;
}

/** One tooltip, with its own provider so it works anywhere. */
export function Tooltip(
	props: React.ComponentProps<typeof TooltipPrimitive.Root>,
) {
	return (
		<TooltipProvider>
			<TooltipPrimitive.Root {...props} />
		</TooltipProvider>
	);
}

/** The element that opens the tooltip. */
export const TooltipTrigger = TooltipPrimitive.Trigger;

/** The note itself. */
export function TooltipContent({
	className,
	sideOffset = 4,
	children,
	...props
}: React.ComponentProps<typeof TooltipPrimitive.Content>) {
	return (
		<TooltipPrimitive.Portal>
			<TooltipPrimitive.Content
				sideOffset={sideOffset}
				className={cn(
					"z-50 max-w-xs rounded-md bg-primary px-3 py-1.5 text-xs text-primary-foreground",
					className,
				)}
				{...props}
			>
				{children}
			</TooltipPrimitive.Content>
		</TooltipPrimitive.Portal>
	);
}
