import { useEffect } from "react";
import { useState } from "react";
import { BehaviorSubject } from "rxjs";

export function useSubject<T>(subject: BehaviorSubject<T>): T {
    const [state, setState] = useState(subject.value)

    useEffect(() => {
        const subscription = subject.subscribe((val) => setState(val))
        return () => subscription.unsubscribe()
    })

    return state
}