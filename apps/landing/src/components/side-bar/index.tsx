'use client'

import { Box, Button, css } from '@devup-ui/react'
import clsx from 'clsx'
import {
  cloneElement,
  ComponentProps,
  createContext,
  Dispatch,
  isValidElement,
  SetStateAction,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react'

import * as keyframes from './keyframes'

const SideBarContext = createContext<{
  isOpen: boolean
  setIsOpen: Dispatch<SetStateAction<boolean>>
} | null>(null)

export function useSideBar() {
  const context = useContext(SideBarContext)
  if (!context) {
    throw new Error('useSideBar must be used within a SideBarProvider')
  }
  return context
}

export function SideBarProvider({ children }: { children: React.ReactNode }) {
  const [isOpen, setIsOpen] = useState(false)
  return (
    <SideBarContext.Provider value={{ isOpen, setIsOpen }}>
      {children}
    </SideBarContext.Provider>
  )
}

export function SideBarTrigger({
  asChild,
  children,
  ...props
}: Omit<ComponentProps<typeof Button<'button'>>, 'as'> & {
  asChild?: boolean
}) {
  const { setIsOpen } = useSideBar()

  if (asChild) {
    const child = isValidElement(children) ? children : null
    if (!child) return null
    return cloneElement(child, {
      onClick: () => setIsOpen((prev) => !prev),
      ...props,
    })
  }

  return (
    <Button
      bg="transparent"
      border="none"
      onClick={() => setIsOpen((prev) => !prev)}
      p="0"
      styleOrder={1}
      {...props}
    >
      {children}
    </Button>
  )
}

const positions = {
  left: css({
    styleOrder: 1,
    top: 0,
    bottom: 0,
    left: 0,
    borderTopRightRadius: '20px',
    borderBottomRightRadius: '20px',
    transform: 'translateX(-100%)',
    boxShadow: '8px 12px 24px 0 #0000001A',
  }),
  right: css({
    styleOrder: 1,
    top: 0,
    bottom: 0,
    right: 0,
    borderTopLeftRadius: '20px',
    borderBottomLeftRadius: '20px',
    transform: 'translateX(100%)',
    boxShadow: '-8px 12px 24px 0 #0000001A',
  }),
  top: css({
    styleOrder: 1,
    left: 0,
    right: 0,
    top: 0,
    borderBottomLeftRadius: '20px',
    borderBottomRightRadius: '20px',
    transform: 'translateY(-100%)',
    boxShadow: '0px 8px 24px 0 #0000001A',
  }),
  bottom: css({
    styleOrder: 1,
    left: 0,
    right: 0,
    bottom: 0,
    borderTopLeftRadius: '20px',
    borderTopRightRadius: '20px',
    transform: 'translateY(100%)',
    boxShadow: '0px -8px 24px 0 #0000001A',
  }),
}
export function SideBarContainer({
  position = 'right',
  className,
  ...props
}: Omit<ComponentProps<typeof Box<'div'>>, 'as'> & {
  position?: 'left' | 'right' | 'top' | 'bottom'
}) {
  const ref = useRef<HTMLDivElement>(null)
  const { isOpen, setIsOpen } = useSideBar()
  const [innerOpen, setInnerOpen] = useState(false)
  const render = isOpen || innerOpen

  useEffect(() => {
    function handleOutsideClick(event: MouseEvent) {
      if (!isOpen) return
      if (ref.current && !ref.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('click', handleOutsideClick)
    return () => document.removeEventListener('click', handleOutsideClick)
  }, [setIsOpen, isOpen])

  return (
    render && (
      <Box
        ref={(node) => {
          if (!node?.checkVisibility()) return () => {}
          ref.current = node
          return () => {
            ref.current = null
          }
        }}
        animationDuration="0.3s"
        animationFillMode="forwards"
        animationTimingFunction="ease-in-out"
        aria-label="side-bar"
        bg="$containerBackground"
        className={clsx(
          positions[position as keyof typeof positions],
          className,
        )}
        onAnimationEnd={() => setInnerOpen(isOpen)}
        pos="fixed"
        px="24px"
        py="30px"
        style={{
          animationName: {
            left: { open: keyframes.leftOpen, close: keyframes.leftClose },
            right: { open: keyframes.rightOpen, close: keyframes.rightClose },
            top: { open: keyframes.topOpen, close: keyframes.topClose },
            bottom: {
              open: keyframes.bottomOpen,
              close: keyframes.bottomClose,
            },
          }[position as keyof typeof positions][isOpen ? 'open' : 'close'],
        }}
        styleOrder={1}
        zIndex={100}
        {...props}
      />
    )
  )
}

export function SideBar() {
  return <></>
}
