import { formatDistanceToNowStrict as formatDistanceToNowStrictFn } from 'date-fns';

export function formatCurrentDistanceDateToNow(date: Date): string {
    return formatDistanceToNowStrictFn(date);
}