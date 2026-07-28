import * as React from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../../lib/utils"
import { useShadowRoot } from "../../contexts/ShadowRootContext";
import { useT } from "@alioth/i18n";

/**
 * Radix UI Dialog 的 TitleWarning / DescriptionWarning 在 Shadow DOM 内会使用
 * document.getElementById() 查找元素，而 Shadow DOM 内的元素无法被 document 访问。
 * 此 hook 在 useLayoutEffect 中检测：若元素 id 在 document 中不存在，
 * 则在 document.body 创建一个等 id 的占位节点，以消除开发时虚假警告。
 * 占位节点 aria-hidden 且视觉隐藏，不影响无障碍。
 */
function useShadowDOMCompat(
  ref: React.RefObject<HTMLElement | null>
) {
  React.useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const id = el.id;
    if (!id) return;
    if (document.getElementById(id)) return;

    const dummy = document.createElement("div");
    dummy.id = id;
    dummy.setAttribute("aria-hidden", "true");
    dummy.style.cssText =
      "position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0;";
    document.body.appendChild(dummy);

    return () => {
      dummy.remove();
    };
  }, []);
}

function composeRefs<T>(
  forwardedRef: React.ForwardedRef<T>,
  innerRef: React.MutableRefObject<T | null>
) {
  return (node: T | null) => {
    innerRef.current = node;
    if (typeof forwardedRef === "function") {
      forwardedRef(node);
    } else if (forwardedRef) {
      forwardedRef.current = node;
    }
  };
}

const StandardDrawer = DialogPrimitive.Root;

const StandardDrawerTrigger = DialogPrimitive.Trigger;

const StandardDrawerClose = DialogPrimitive.Close;

const StandardDrawerPortal = ({
  children,
  ...props
}: React.ComponentPropsWithoutRef<typeof DialogPrimitive.Portal>) => {
  const shadow = useShadowRoot();
  return (
    <DialogPrimitive.Portal container={shadow ?? undefined} {...props}>
      {children}
    </DialogPrimitive.Portal>
  );
};

const StandardDrawerOverlay = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Overlay>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay>
>(({ className, style, ...props }, ref) => (
  <DialogPrimitive.Overlay
    className={cn(
      "fixed inset-0 z-50 bg-black/60 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
      className,
    )}
    style={{
      ...style,
      right: "var(--standard-drawer-right, 0px)",
    }}
    {...props}
    ref={ref}
  />
));
StandardDrawerOverlay.displayName = DialogPrimitive.Overlay.displayName;

const standardDrawerVariants = cva(
  "fixed z-50 gap-4 bg-background shadow-xl transition ease-in-out data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:duration-300 data-[state=open]:duration-500",
  {
    variants: {
      side: {
        right:
          "top-0 bottom-0 h-full border-l data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right",
      },
    },
    defaultVariants: {
      side: "right",
    },
  },
);

interface StandardDrawerContentProps
  extends React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content>,
    VariantProps<typeof standardDrawerVariants> {
  width?: number;
  hideCloseButton?: boolean;
  /** 右侧偏移量（覆盖 CSS 变量）。由 ModuleLayout 等布局组件自动注入。 */
  rightOffset?: string | number;
}

const StandardDrawerContent = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  StandardDrawerContentProps
>(
  (
    { side = "right", width = 480, hideCloseButton = false, rightOffset, className, children, ...props },
    ref,
  ) => {
    const t = useT();
    const rightValue = rightOffset !== undefined
      ? typeof rightOffset === "number" ? `${rightOffset}px` : rightOffset
      : "var(--standard-drawer-right, 0px)";

    const handleInteractOutside = (event: CustomEvent) => {
      const originalEvent = event.detail?.originalEvent;
      const target = originalEvent?.target ?? event.target;
      if (target instanceof Element && target.closest("[data-right-sidebar]")) {
        event.preventDefault();
      }
    };

    return (
    <StandardDrawerPortal>
      <StandardDrawerOverlay />
      <DialogPrimitive.Content
        ref={ref}
        className={cn(standardDrawerVariants({ side }), "flex flex-col p-0", className)}
        style={{
          width: `${width}px`,
          maxWidth: "100vw",
          right: rightValue,
        }}
        onPointerDownOutside={handleInteractOutside}
        onInteractOutside={handleInteractOutside}
        {...props}
      >
        <div className="flex flex-1 flex-col overflow-y-auto min-h-0">
          {children}
        </div>
        {!hideCloseButton && (
          <DialogPrimitive.Close className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-secondary">
            <X className="h-4 w-4" />
            <span className="sr-only">{t("components.close")}</span>
          </DialogPrimitive.Close>
        )}
      </DialogPrimitive.Content>
    </StandardDrawerPortal>
  );
});
StandardDrawerContent.displayName = DialogPrimitive.Content.displayName;

const StandardDrawerHeader = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      "flex flex-col space-y-2 text-center sm:text-left",
      className,
    )}
    {...props}
  />
);
StandardDrawerHeader.displayName = "StandardDrawerHeader";

const StandardDrawerFooter = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2",
      className,
    )}
    {...props}
  />
);
StandardDrawerFooter.displayName = "StandardDrawerFooter";

const StandardDrawerTitle = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Title>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Title>
>(({ className, ...props }, forwardedRef) => {
  const innerRef = React.useRef<HTMLHeadingElement>(null);
  useShadowDOMCompat(innerRef);

  return (
    <DialogPrimitive.Title
      ref={composeRefs(forwardedRef, innerRef)}
      className={cn("text-lg font-semibold text-foreground", className)}
      {...props}
    />
  );
});
StandardDrawerTitle.displayName = DialogPrimitive.Title.displayName;

const StandardDrawerDescription = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Description>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Description>
>(({ className, ...props }, forwardedRef) => {
  const innerRef = React.useRef<HTMLParagraphElement>(null);
  useShadowDOMCompat(innerRef);

  return (
    <DialogPrimitive.Description
      ref={composeRefs(forwardedRef, innerRef)}
      className={cn("text-sm text-muted-foreground", className)}
      {...props}
    />
  );
});
StandardDrawerDescription.displayName = DialogPrimitive.Description.displayName;

export {
  StandardDrawer,
  StandardDrawerPortal,
  StandardDrawerOverlay,
  StandardDrawerTrigger,
  StandardDrawerClose,
  StandardDrawerContent,
  StandardDrawerHeader,
  StandardDrawerFooter,
  StandardDrawerTitle,
  StandardDrawerDescription,
};
