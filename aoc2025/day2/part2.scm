(begin
    (define pos 50)
    (define inputs (split-string-with (read-file "aoc2025/day2/input") ","))
    (define (parse-input s)
        (define strings (split-string-with s "-"))
        (map string->int strings)
    )
    (define inputs (map parse-input inputs))


    (define (rev-range from)
        (make-generator from -1)
    )

    (define (repeats? string n)
        (define len (length string))
        (if (= (mod len n) 0)
            (begin
                (define first (substring string 0 n))
                ; (display "\n" string " first:" first "\n")
                (define starts (make-generator n n))
                (define different-spotted #f)
                (do
                    ((start (starts) (starts)))
                    (
                        (or different-spotted (> (+ start n) len))
                        (not different-spotted)
                    )
                    (define current (substring string start (+ start n)))
                    ; (display " -> current:" current "\n")
                    (if (not (= current first))
                        (set! different-spotted #t)
                        #f
                    )
                )
            )

            #f
        )
    )

    (define (invalid-id? id)
        (define s (->string id))
        (define len (length s))
        (define ns (rev-range (/ len 2)))
        (define invalid #f)
        (do
            ((n (ns) (ns)))
            ((or invalid (= n 0)) invalid)
            (if
                (repeats? s n)
                (begin
                    (display "==> " id " is invalid\n")
                    (set! invalid #t)
                )
                #f
            )
        )
    )

    (define invalid-ids 0)
    (define (count-invalid-ids start stop)
        (define generator (make-generator start 1))
        (do
            ((val (generator) (generator)))
            ((> val stop))

            (if (invalid-id? val)
                (set! invalid-ids (+ invalid-ids val))
                #f
            )
        )
    )

    (map (lambda (input) (apply count-invalid-ids input)) inputs)
    (display invalid-ids)
)
