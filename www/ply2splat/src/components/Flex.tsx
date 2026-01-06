import { cn } from "@/lib/utils";

type FlexDirection = "row" | "col";
export function Flex({
	direction = "row",
	className,
	children,
	...props
}: { direction?: FlexDirection } & React.PropsWithChildren<
	React.HTMLProps<HTMLDivElement>
>) {
	return (
		<div
			className={cn("flex", direction === "col" && "flex-col", className)}
			{...props}
		>
			{children}
		</div>
	);
}
