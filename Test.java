import java.lang.*;
import java.util.Arrays;
class SystemDemo 
{
    public static void main(String args[])
    {
        int[] a = {1, 2, 3, 4, 5};
        int[] b = {6, 7, 8, 9, 10};

        System.arraycopy(a, 0, b, 2, 2);

        // array b after arraycopy operation
        System.out.println(Arrays.toString(b));

    }
}
