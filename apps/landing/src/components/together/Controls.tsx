import { css } from '@devup-ui/react'
import type { InputHTMLAttributes, ReactNode } from 'react'

const choice = css({
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  position: 'relative',
  border: '1px solid $primary',
  borderRadius: '1000px',
  px: ['19px', null, null, '27px'],
  py: ['7px', null, null, '5px'],
  color: '$primary',
  typography: 'body',
  cursor: 'pointer',
  selectors: {
    '&:has(input[type="checkbox"])': {
      px: ['16px', null, null, '20px'],
      py: ['10px', null, null, '6px'],
    },
    '&:has(input[type="checkbox"]:checked)': {
      pl: ['12px', null, null, '14px'],
      pr: ['16px', null, null, '20px'],
      gap: '6px',
    },
    '&:has(input[type="checkbox"]:checked)::before': {
      content: '""',
      display: 'inline-block',
      width: '16px',
      height: '16px',
      backgroundColor: '$base',
      maskImage: [
        'url(/images/together/selected-mobile.svg)',
        null,
        null,
        'url(/images/together/selected-pc.svg)',
      ],
      maskPosition: 'center',
      maskRepeat: 'no-repeat',
      maskSize: 'contain',
    },
    '&:has(input:checked)': { bg: '$primary', color: '$base' },
    '&:has(input[aria-invalid="true"])': { border: '2px solid $togetherError' },
    '&:has(input:focus-visible)': {
      boxShadow: '0 0 0 5px $primary, 0 0 0 2px $base',
    },
    '&:hover:not(:has(input:checked))': { bg: '$menuHover' },
  },
})

const hiddenInput = css({
  position: 'absolute',
  width: '1px',
  height: '1px',
  opacity: 0,
})

export function Controls({
  children,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { children: ReactNode }) {
  return (
    <label className={choice}>
      <input className={hiddenInput} {...props} />
      {children}
    </label>
  )
}

export const field = css({
  appearance: 'none',
  width: '100%',
  minHeight: ['48px', null, null, '52px'],
  padding: ['12px', null, null, '13px 16px'],
  border: '1px solid $primary',
  borderRadius: '12px',
  background: '$containerBackground',
  color: '$text',
  typography: 'body',
  _focusVisible: { outline: '2px solid $link', outlineOffset: '2px' },
  selectors: {
    '&[data-registration-field]': {
      fontSize: '16px',
      lineHeight: ['normal', null, null, '1.8'],
    },
    '&::placeholder': {
      color: '$togetherPlaceholder',
      opacity: 1,
    },
    '&[aria-invalid="true"]': { border: '2px solid $togetherError' },
    '&[data-multiline]': {
      height: ['120px', null, null, '140px'],
      resize: 'vertical',
    },
  },
})

export const fieldLabel = css({
  display: 'flex',
  flexDirection: 'column',
  gap: '8px',
  color: '$title',
  typography: 'bodyBold',
})

export const errorMessage = css({
  display: 'inline-flex',
  alignItems: 'center',
  gap: '6px',
  color: '#C8124B',
  _themeDark: { color: '$title' },
  typography: 'docsCaption',
  selectors: {
    '&::before': {
      content: '""',
      flexShrink: 0,
      width: '16px',
      height: '16px',
      backgroundColor: 'currentColor',
      maskImage: 'url(/images/together/field-alert.svg)',
      maskPosition: 'center',
      maskRepeat: 'no-repeat',
      maskSize: 'contain',
    },
  },
})

export const consentControl = css({
  appearance: 'none',
  flexShrink: 0,
  width: '24px',
  height: '24px',
  border: '1.5px solid $primary',
  borderRadius: '5px',
  background: '$containerBackground',
  cursor: 'pointer',
  _checked: { bg: '$primary', color: '$base' },
  _focusVisible: { outline: '2px solid $link', outlineOffset: '2px' },
  selectors: {
    '&[aria-invalid="true"]': { border: '2px solid $togetherError' },
    '&:checked::after': {
      content: '"✓"',
      display: 'block',
      textAlign: 'center',
      fontSize: '18px',
      lineHeight: '20px',
    },
  },
})

export const primaryAction = css({
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  border: '1px solid $primary',
  borderRadius: '1000px',
  padding: '17px 32px',
  background: '$primary',
  color: '$base',
  typography: 'button',
  cursor: 'pointer',
  textDecoration: 'none',
  _hover: { opacity: 0.8 },
  _focusVisible: { outline: '2px solid $link', outlineOffset: '2px' },
})

export const secondaryAction = css({
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  border: '1px solid $primary',
  borderRadius: '1000px',
  padding: '17px 32px',
  background: '$containerBackground',
  color: '$text',
  typography: 'button',
  cursor: 'pointer',
  textDecoration: 'none',
  _hover: { bg: '$menuHover' },
  _focusVisible: { outline: '2px solid $link', outlineOffset: '2px' },
})

export const retryAction = css({
  border: 'none',
  padding: '0',
  background: 'transparent',
  color: '$text',
  typography: 'docsCaption',
  textDecoration: 'underline',
  cursor: 'pointer',
  _focusVisible: { outline: '2px solid $link', outlineOffset: '2px' },
})
